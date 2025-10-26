# Implementation Plan – Search Overhaul

Last updated: October 23, 2025  
Context: reworking Nexus search to deliver hybrid lexical + semantic relevance with fresh embeddings, moving embedding inference to a dedicated service, and tightening the ingestion/runtime story ahead of v0.2 launch.

**Update – October 24, 2025:** Semantic search work is paused while we focus on infrastructure readiness. The backend has reverted to lexical-only search, embeddings remain in the schema for future use, and UI/admin controls for embeddings are hidden. The open items below that reference embeddings are deferred until we restart the effort.

**Update – October 25, 2025:** We are tuning lexical search to handle ~80 GB of mailing list content efficiently. Focus areas: eliminate git patch noise from FTS inputs, add optional `start_date`/`end_date` filters on the thread search endpoint, and adjust ranking to weight subjects and thread-starter messages most heavily with a recency boost tie-breaker.

---

## Goals

- Adopt `nomic-ai/nomic-embed-text-v1.5` as the semantic backbone, choosing a default dimensionality that balances quality and storage (target 768-D per design).
- Serve embeddings via a first-class Docker Compose service so the API server no longer loads models in-process and can scale independently.
- Normalize hybrid search: consistent prefixing for query/document embeddings, pgvector-friendly storage, resilient fallbacks, and admin tooling that keeps indexes fresh.
- Precompute thread-level embeddings in the background as new emails land so hybrid search is always warm without blocking ingestion.
- Elevate UX: expose lexical-only, semantic-only, and hybrid modes with clear toggles and score context in the UI.
- Keep operational posture tight: metrics, retries, migrations, docs, and local dev ergonomics all ready before rollout.
- Move embedding inference and index maintenance onto a durable job queue so imports finish quickly while follow-up search work runs asynchronously with visible status and retry semantics.
- Deliver fast lexical search over large corpora by stripping git patch payloads from FTS documents, weighting thread subjects and first-post discussions higher than later replies, exposing optional date range filters, and adding a lightweight recency boost to the ranking pipeline.

## Non-Goals

- Reranking is out of scope for now; we will not ship cross-encoder infrastructure in this iteration.
- No change to Keycloak/OIDC auth scope beyond what search endpoints already expose.
- No rewrite of grokmirror ingestion logic outside embedding/fts touchpoints.

## Current State Snapshot

- API exposes `/threads/search` and `/authors/search` using SQL `LIKE` over subjects/bodies; embeddings column (`emails.embedding VECTOR(384)`) exists but is unused. (`api-server/src/routes/threads.rs`, `authors.rs`)
- `docs/design.md` promises hybrid search + embeddings, but operational details (model choice, service, data flow) are incomplete.
- Compose stack has Postgres, API, frontend, keycloak; no dedicated ML service today.

## Key Decisions & Architecture Notes

- **Embedding dimensionality**: migrate `emails.embedding` to `VECTOR(768)` so we keep the full-quality representation while allowing matryoshka truncation later if needed.
- **Thread embeddings only**: compute embeddings for each email and aggregate per-thread vectors asynchronously as new content arrives; author search remains lexical-only.
- **Inference service**: run Hugging Face Text Embeddings Inference (`ghcr.io/huggingface/text-embeddings-inference:cpu-1.8` by default) with `--model-id nomic-ai/nomic-embed-text-v1.5`; expose HTTP on `http://embeddings:8080`.
- **Patch stripping**: normalize email bodies by removing patch hunks/attachments before embedding to focus on discussion text.
- **Prefix discipline**: prepend `search_document:` for corpus items and `search_query:` for user queries prior to inference to match model expectations.
- **TEI concurrency**: expose `EMBEDDINGS_MAX_CONCURRENT_REQUESTS` so deployments (especially GPU-backed) can dial in router parallelism without code changes.
- **Batching & retries**: ingestion pipeline sends batches (size configurable) to the service; implement exponential backoff and circuit breaker before falling back to lexical-only search.
- **Feature flags**: gate semantic scoring behind a config switch (`SEARCH_ENABLE_VECTOR`, `EMBEDDING_DIMENSION`) so we can roll out gradually.
- **Metrics & health**: embed latency histograms, per-batch failure counters, and a Rocket readiness check that fails fast if the embedding service is unavailable beyond a configurable threshold.
- **Job queue standardisation**: promote `sync_jobs` to a typed job queue with `job_type = {import, embedding-refresh, index-maintenance}` and unified status vocabulary (`queued`, `running`, `succeeded`, `failed`, `cancelled`) exposed through admin APIs and UI.
- **Admin control surface**: provide explicit endpoints to drop embeddings, rebuild indexes, and schedule per-mailing-list or global refreshes so operators can trigger remediation without touching the database manually.
- **Thread search vector**: introduce a derived `threads.search_vector` (or equivalent view) composed via `setweight`—subject lines weigh `A`, the first email discussion weighs `B`, and subsequent replies weigh `D`. All text runs through the patch-stripper so diff/trailer ranges never enter the tsvector.
- **Discussion sanitizer**: rely on `PatchMetadata.diff_sections`/`trailer_sections` recorded during parsing to drop inline diffs, trailer blocks, and diffstats when generating full-text fields. Fallback to heuristics for legacy rows missing metadata.
- **Date scoping**: extend `/threads/search` params with optional `start_date`/`end_date` (UTC ISO dates). Default to the full archive when unset; guard invalid ranges and add OpenAPI coverage.
- **Recency boost**: blend lexical rank with a decay factor such as `exp(-GREATEST(0, EXTRACT(EPOCH FROM NOW() - t.last_date)) / recency_half_life)` so recent threads tie-break older hits without overwhelming strong lexical matches.
- **Supporting indexes**: add/confirm `threads (mailing_list_id, last_date DESC)` and consider covering indexes on `(mailing_list_id, start_date, id)` to keep date-bounded queries fast.
- **Bulk refresh workflow**: extend search maintenance jobs to rebuild the new sanitized vectors after imports or when patch metadata backfills complete.

### Lexical Sanitization Strategy

- Implement a reusable Rust helper `strip_patch_payload(body: &str, metadata: Option<&PatchMetadata>) -> Cow<'_, str>` that removes line ranges identified by `diff_sections`, `diffstat_section`, and `trailer_sections`; drop entire bodies when `is_patch_only` is `true`.
- When `PatchMetadata` is absent but `patch_type != PatchType::None`, fall back to parser heuristics (diff regex, trailer scan) to avoid indexing raw patches for legacy rows.
- Preserve conversational quoting (`>`-prefixed) and inline replies while collapsing excessive blank lines during reconstruction so `tsvector` density stays high without altering message meaning.
- Store the sanitized discussion text alongside original email data (`emails.search_body` or equivalent view) so both ingestion and backfill paths can call the same helper before refreshing FTS columns.

---

## Checklist

1. **Discovery & Design Updates**
   - [x] Confirm current SQL schema vs design (vector column, indexes, extensions) and document any drift.
   - [x] Update `docs/design.md` search sections with the finalized embedding dimensionality, service topology, and prefix rules.
   - [ ] Capture data retention considerations (vector storage growth, re-embed cadence) in the design doc.
   - [x] Document the queue-driven embedding refresh flow and new admin endpoints in both design doc and API reference notes.

2. **Database & Migrations**
   - [x] Write reversible migration to alter `emails.embedding` to `VECTOR(768)` and refresh associated indexes (HNSW + support).  
   - [x] Add thread-level embedding materialization table/view if needed (e.g., `thread_embeddings`) to accelerate lookup.
   - [ ] Extend repeatable migration to assert `CREATE EXTENSION IF NOT EXISTS vectorchord` still succeeds post-dimension change and ensure down migration notes dimensionality limitations.
   - [x] Introduce `job_type` enum plus `status` + `payload` columns on `sync_jobs` (or successor table) to support embedding/index work units and consistent lifecycle timestamps.
   - [ ] Add search-maintenance specific tables if needed (e.g., `embedding_refresh_cursor`) to enable idempotent incremental rebuilds.
   - [x] Add `emails.search_body TEXT` (nullable, toasted) populated by the sanitizer; keep `emails.body` untouched for raw rendering.
   - [x] Add `threads.search_vector TSVECTOR` plus `CREATE INDEX idx_threads_search_vector ON threads USING GIN (search_vector)`.
   - [ ] Extend repeatable migration / triggers so thread inserts/updates refresh `search_vector` via sanitized email text and subject weighting.
   - [x] Ensure `idx_threads_last_date` covers the recency-sort path; add composite index `(mailing_list_id, last_date DESC)` if planner regressions show up.

3. **Embedding Service Integration** _(Deferred)_
   - [x] Add an `embeddings` service to `docker-compose.yml` using the TEI image, mounting a cache volume and wiring `HF_TOKEN` passthrough when needed.
   - [x] Define healthcheck (`GET /health`) and align Compose depends_on/condition so API waits for the service.
   - [ ] Provide local overrides for GPU users (notes on alternative TEI images) without breaking CPU default.
   - [x] Document environment knobs: `EMBEDDINGS_URL`, `EMBEDDINGS_MODEL_ID`, `EMBEDDINGS_DIM`.

4. **API Ingestion & Sync** _(Deferred semantic pieces)_
   - [ ] (Deferred) Implement embedding client module (Tokio HTTP, connection pooling) with request/response schema matching TEI `/embed`.
   - [ ] (Deferred) Extend sync/import pipeline to batch canonical email content, call the service, and persist email vectors alongside lexical artifacts.
   - [ ] (Deferred) Add background job to aggregate/upsert thread embeddings whenever new emails arrive or existing emails change.
   - [ ] (Deferred) Build retry/backoff strategy and dead-letter logging for failed embedding batches; escalate after threshold.
   - [ ] (Deferred) Split ingestion flow so sync/import jobs enqueue embedding work instead of embedding inline once the service returns.
   - [ ] (Deferred) Record import completion metadata that embedding jobs can use to scope missing-vector queries (e.g., `imported_email_ids`, `last_processed_date`).
   - [ ] (Deferred) Reintroduce admin jobs (`/admin/search/embeddings/rebuild`) when semantic refreshes resume.

5. **Search Query Path** _(Lexical live; semantic deferred)_
   - [x] Maintain lexical search endpoint with REST-friendly request/response shapes.
   - [ ] (Deferred) Reintroduce semantic query path (KNN via `<=>`) and hybrid fusion once embeddings return.
   - [x] Ensure lexical-only author search remains unchanged and leverages shared query params.
   - [ ] (Deferred) Restore fallback logic and warnings once semantic search is back.
   - [x] Update API response schema to reflect lexical-only results (mode field removed).
   - [ ] (Deferred) Add integration tests covering mode toggles, empty embeddings, and service outages when semantic modes return.
   - [x] Add optional `start_date` / `end_date` filters to `ThreadSearchParams`, enforce validation, and document defaults in OpenAPI + `docs/design.md`.
   - [x] Build a sanitization helper (Rust) that consumes `PatchMetadata` to remove diff/diffstat/trailer lines from email bodies before they feed FTS columns.
   - [ ] Recompute stored `tsvector` data (new `threads.search_vector` or refreshed `lex_ts`) using weighted `setweight` composition: subject (`A`), thread-starter body (`B`), remaining replies (`D`).
   - [x] Update the SQL ranking pipeline to blend weighted lexical scores with a recency decay factor; benchmark alternative decay constants against sample queries.
   - [ ] Confirm email ingestion keeps `PatchMetadata` populated (add backfill or guardrails for legacy rows) so sanitization stays deterministic.
   - [ ] Add integration coverage that exercises date filters, subject weighting, and recency tie-breaking on representative fixtures.

```sql
-- Weighted lexical search with optional date bounds and recency decay.
WITH query AS (
    SELECT websearch_to_tsquery('english', $2) AS tsq,
           COALESCE($7::double precision, 31 * 24 * 3600) AS half_life_seconds
),
filtered_threads AS (
    SELECT t.*
    FROM threads t
    WHERE t.mailing_list_id = $1
      AND ($5 IS NULL OR t.last_date >= $5)
      AND ($6 IS NULL OR t.start_date <= $6)
),
thread_docs AS (
    SELECT
        t.id,
        setweight(to_tsvector('english', COALESCE(t.subject, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(fe.search_body, '')), 'B') ||
        setweight(to_tsvector('english', COALESCE(rest.tail_text, '')), 'D') AS search_vector,
        t.last_date
    FROM filtered_threads t
    LEFT JOIN emails fe
        ON fe.message_id = t.root_message_id
       AND fe.mailing_list_id = t.mailing_list_id
    LEFT JOIN LATERAL (
        SELECT string_agg(e.search_body, ' ' ORDER BY e.date) AS tail_text
        FROM thread_memberships tm
        JOIN emails e ON tm.email_id = e.id
        WHERE tm.thread_id = t.id
          AND tm.mailing_list_id = t.mailing_list_id
          AND e.id <> fe.id
    ) rest ON TRUE
),
ranked AS (
    SELECT
        td.id,
        ts_rank_cd(td.search_vector, query.tsq) AS text_score,
        exp(-GREATEST(0, EXTRACT(EPOCH FROM ((NOW() AT TIME ZONE 'utc') - td.last_date))) / query.half_life_seconds) AS recency_factor,
        td.last_date
    FROM thread_docs td
    CROSS JOIN query
    WHERE td.search_vector @@ query.tsq
)
SELECT
    r.id,
    (r.text_score * (1.0 + $8::double precision * r.recency_factor)) AS blended_score,
    r.text_score,
    r.recency_factor,
    r.last_date
FROM ranked r
ORDER BY blended_score DESC, r.last_date DESC
LIMIT $3 OFFSET $4;
```

6. **Frontend Experience** _(Semantic UI deferred)_
   - [x] Simplify the search UI for lexical-only mode and remove semantic toggles until the feature returns.
   - [ ] (Deferred) Surface semantic service status/fallback messaging when semantic search is re-enabled.
   - [ ] (Deferred) Re-add admin controls for embedding maintenance once backfills are possible.
   - [x] Align job status chips/labels in the UI with the normalized server vocabulary (`queued`, `running`, `succeeded`, `failed`, `cancelled`).
   - [ ] Add analytics/logging hook (if available) to track search usage.

7. **Ops, Observability & Tooling**
   - [ ] (Deferred) Expose Prometheus metrics: embedding request latency, batch size, retry count, vector vs lexical usage split.
   - [ ] (Deferred) Wire logs to include embedding service status in health output (`/health`, `/metrics`).
   - [ ] (Deferred) Add Makefile target (`make embeddings-shell` or similar) to exec into the service for debugging.
   - [ ] Provide load test plan (e.g., k6 script) to validate throughput under the lexical-only workload; extend for TEI once reinstated.
   - [ ] Emit queue depth / job age metrics for import jobs (embedding jobs deferred) and add alerting guidance for stuck jobs.
   - [ ] Document operational runbooks for admin endpoints (drop/regenerate indexes) including expected runtime impact.
   - [ ] Capture benchmarks for the sanitized FTS pipeline (index build time, query latency with/without date filters) and record tuning guidance for `WORK_MEM`/GIN maintenance.

8. **Testing & QA**
   - [ ] (Deferred) Unit-test embedding client (prefix handling, dimensionality guard, error unwrap).
   - [ ] Integration test end-to-end lexical search (seed data, ensure ordering behaves as expected).
   - [ ] Document manual QA script: bootstrap DB, run sync subset, verify Compose workflow (`make up`, `npm run dev`, search flows).
   - [ ] (Deferred) Add Playwright coverage for semantic mode toggles when they return.
   - [ ] (Deferred) Add integration coverage for the job queue dispatcher once embedding jobs are reinstated.
   - [ ] Update API contract tests for index reset endpoints; embedding endpoints remain deferred.
   - [ ] Add regression fixtures ensuring patch stripping preserves conversational text while removing diffs, and that recency decays never promote empty hits.

9. **Documentation & Rollout**
   - [ ] Update `docs/design.md` and `README.md` sections on search setup, including Compose instructions and troubleshooting.
   - [ ] Prepare migration/upgrade notes (e.g., “rerun `make init` to pull new service”).
   - [ ] Draft release checklist covering data backfills, downtime expectations, and toggle plan.
   - [ ] Publish admin playbook detailing queue usage, job monitoring, and recovery steps for failed embedding batches.

---

## Testing Strategy

- **Automated**: Focus on Rust integration tests for lexical scoring; defer TEI mock coverage until semantic search resumes.
- **Manual**: Compose stack smoke test, health endpoint verification, sample query comparisons (before vs after), and load sampling to ensure latency targets (<300 ms for top-K 50 on CPU).
- **Backfill rehearsal**: (Deferred) Dry-run re-embedding on a staging snapshot to size runtime and validate migration rollbacks.

## Risks & Mitigations

- **Service availability**: (Deferred) Semantic service outage risk returns once embeddings are enabled. Current lexical-only stack has no external inference dependency.
- **Storage bloat**: 768-D doubles vector storage vs 384-D; monitor table growth even while columns stay NULL, and consider matryoshka truncation if/when embeddings populate.
- **Prefix misuse**: (Deferred) Missing `search_query:` / `search_document:` prefixes would degrade semantic quality; keep helpers ready for when embeddings return.
- **Patch stripping regressions**: If patch detection removes relevant conversational content, search quality drops; maintain heuristics and tests over sample emails.
- **Latency on CPU**: (Deferred) TEI CPU image may struggle under load; provide GPU path and consider caching hot embeddings when workload is re-enabled.
- **Queue drift**: (Deferred) If embedding jobs stall, imports might appear complete while semantic search lags; surface queue health metrics and expose manual recovery APIs.
- **Operational misuse**: Dropping indexes on prod during peak hours could spike load; document guardrails and require confirmation tokens in admin UI.
- **Recency overweighting**: Aggressive decay constants could bury historically important threads; log parameter defaults, expose configuration, and add analytics to monitor click-through on older results.

## Open Questions

- Should we store multiple embedding dimensionalities (e.g., 768 + 256) for tiered search, or truncate on-the-fly?
- What telemetry do we need to decide if/when to enable reranking?
- Do we introduce priority lanes for embedding jobs sourced from user-triggered rebuilds vs automated post-import refreshes?
- How do we authenticate UI-triggered destructive actions (drop all embeddings) to avoid accidental clicks? Two-step confirmation or admin PIN?

---

Archive this plan once the overhaul ships and note the final completion date.

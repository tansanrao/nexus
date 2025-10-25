# Implementation Plan – Search Overhaul

Last updated: October 23, 2025  
Context: reworking Nexus search to deliver hybrid lexical + semantic relevance with fresh embeddings, moving embedding inference to a dedicated service, and tightening the ingestion/runtime story ahead of v0.2 launch.

**Update – October 24, 2025:** Semantic search work is paused while we focus on infrastructure readiness. The backend has reverted to lexical-only search, embeddings remain in the schema for future use, and UI/admin controls for embeddings are hidden. The open items below that reference embeddings are deferred until we restart the effort.

---

## Goals

- Adopt `nomic-ai/nomic-embed-text-v1.5` as the semantic backbone, choosing a default dimensionality that balances quality and storage (target 768-D per design).
- Serve embeddings via a first-class Docker Compose service so the API server no longer loads models in-process and can scale independently.
- Normalize hybrid search: consistent prefixing for query/document embeddings, pgvector-friendly storage, resilient fallbacks, and admin tooling that keeps indexes fresh.
- Precompute thread-level embeddings in the background as new emails land so hybrid search is always warm without blocking ingestion.
- Elevate UX: expose lexical-only, semantic-only, and hybrid modes with clear toggles and score context in the UI.
- Keep operational posture tight: metrics, retries, migrations, docs, and local dev ergonomics all ready before rollout.
- Move embedding inference and index maintenance onto a durable job queue so imports finish quickly while follow-up search work runs asynchronously with visible status and retry semantics.

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

8. **Testing & QA**
   - [ ] (Deferred) Unit-test embedding client (prefix handling, dimensionality guard, error unwrap).
   - [ ] Integration test end-to-end lexical search (seed data, ensure ordering behaves as expected).
   - [ ] Document manual QA script: bootstrap DB, run sync subset, verify Compose workflow (`make up`, `npm run dev`, search flows).
   - [ ] (Deferred) Add Playwright coverage for semantic mode toggles when they return.
   - [ ] (Deferred) Add integration coverage for the job queue dispatcher once embedding jobs are reinstated.
   - [ ] Update API contract tests for index reset endpoints; embedding endpoints remain deferred.

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

## Open Questions

- Should we store multiple embedding dimensionalities (e.g., 768 + 256) for tiered search, or truncate on-the-fly?
- What telemetry do we need to decide if/when to enable reranking?
- Do we introduce priority lanes for embedding jobs sourced from user-triggered rebuilds vs automated post-import refreshes?
- How do we authenticate UI-triggered destructive actions (drop all embeddings) to avoid accidental clicks? Two-step confirmation or admin PIN?

---

Archive this plan once the overhaul ships and note the final completion date.

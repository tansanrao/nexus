# Implementation Plan – Search Overhaul

Last updated: October 23, 2025  
Context: reworking Nexus search to deliver hybrid lexical + semantic relevance with fresh embeddings, moving embedding inference to a dedicated service, and tightening the ingestion/runtime story ahead of v0.2 launch.

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

3. **Embedding Service Integration**
   - [x] Add an `embeddings` service to `docker-compose.yml` using the TEI image, mounting a cache volume and wiring `HF_TOKEN` passthrough when needed.
   - [x] Define healthcheck (`GET /health`) and align Compose depends_on/condition so API waits for the service.
   - [ ] Provide local overrides for GPU users (notes on alternative TEI images) without breaking CPU default.
   - [x] Document environment knobs: `EMBEDDINGS_URL`, `EMBEDDINGS_MODEL_ID`, `EMBEDDINGS_DIM`.

4. **API Ingestion & Sync**
   - [x] Implement embedding client module (Tokio HTTP, connection pooling) with request/response schema matching TEI `/embed`.
   - [x] Extend sync/import pipeline to batch canonical email content, strip patches, apply prefixes, call service, and store email vectors + `lex_ts`/`body_ts` updates in one transaction.
   - [x] Add background job to aggregate/upsert thread embeddings whenever new emails arrive or existing emails change.
   - [ ] Build retry/backoff strategy and dead-letter logging for failed batches; escalate after threshold.
   - [x] Split ingestion flow so sync/import jobs enqueue `embedding-refresh` work instead of embedding inline; dispatcher claims follow-up jobs and populates vectors before marking import complete.
   - [ ] Record import completion metadata that embedding jobs can use to scope missing-vector queries (e.g., `imported_email_ids`, `last_processed_date`).
   - [x] Add admin job (`/admin/search/embeddings/rebuild`) to re-embed slices (per list, date range) for backfills and wire it to the new queue architecture.

5. **Search Query Path**
   - [x] Implement endpoints supporting `mode=lexical|semantic|hybrid` with REST-friendly request/response shapes aligned with other APIs.
   - [x] Build semantic query path (KNN via `<=>`) using thread embeddings; fuse with FTS score only in hybrid mode.
   - [x] Ensure lexical-only author search remains unchanged but leverages any new shared query params.
   - [x] Add fallback path when embeddings missing (e.g., lexical only with warning header).
   - [x] Update API response schema to include match breakdown (lexical score, semantic score) and mode echoes.
   - [ ] Add integration tests covering mode toggles, empty embeddings, and service outages (mock client).

6. **Frontend Experience**
   - [x] Update `frontend-new` search UI to show semantic relevance indicators and allow toggling between lexical-only, semantic-only, and hybrid modes.
   - [x] Surface loading/error states when semantic service is unavailable; include copy that references automatic fallback.
   - [x] Rework the admin settings database panel to call the new queue-powered endpoints (drop/reset/rebuild embeddings and indexes) with confirmation UX and live job status.
   - [x] Align job status chips/labels in the UI with the normalized server vocabulary (`queued`, `running`, `succeeded`, `failed`, `cancelled`).
   - [ ] Add analytics/logging hook (if available) to track new search usage.

7. **Ops, Observability & Tooling**
   - [ ] Expose Prometheus metrics: embedding request latency, batch size, retry count, vector vs lexical usage split.
   - [ ] Wire logs to include embedding service status in health output (`/health`, `/metrics`).
   - [ ] Add Makefile target (`make embeddings-shell` or similar) to exec into the service for debugging.
   - [ ] Provide load test plan (e.g., k6 script) to validate throughput with TEI CPU container.
   - [ ] Emit queue depth / job age metrics for both import and embedding jobs; add alerting guidance for stuck jobs.
   - [ ] Document operational runbooks for admin endpoints (drop/regenerate vectors, full reset) including expected runtime impact.

8. **Testing & QA**
   - [ ] Unit-test embedding client (prefix handling, dimensionality guard, error unwrap).
   - [ ] Integration test end-to-end search (seed data, ensure lexical-only vs semantic-only vs hybrid ordering behaves as expected).
   - [ ] Document manual QA script: bootstrap DB, run sync subset, verify Compose workflow (`make up`, `npm run dev`, search flows).
   - [x] Add Playwright smoke test covering search mode toggles and UI summary.
   - [ ] Add integration coverage for the job queue dispatcher: import job enqueues embedding job, embedding job updates missing vectors, status transitions stay consistent.
   - [ ] Add API contract tests for the new admin endpoints (drop/rebuild embeddings, index reset) and ensure idempotency under retry.

9. **Documentation & Rollout**
   - [ ] Update `docs/design.md` and `README.md` sections on search setup, including Compose instructions and troubleshooting.
   - [ ] Prepare migration/upgrade notes (e.g., “rerun `make init` to pull new service”).
   - [ ] Draft release checklist covering data backfills, downtime expectations, and toggle plan.
   - [ ] Publish admin playbook detailing queue usage, job monitoring, and recovery steps for failed embedding batches.

---

## Testing Strategy

- **Automated**: Extend existing Rust integration tests (Testcontainers) to start TEI mock (or recorded responses) and assert hybrid scoring. Add frontend Vitest coverage for the new UI toggles.
- **Manual**: Compose stack smoke test, health endpoint verification, sample query comparisons (before vs after), and load sampling to ensure latency targets (<300 ms for top-K 50 on CPU).
- **Backfill rehearsal**: Dry-run re-embedding on a staging snapshot to size runtime and validate migration rollbacks.

## Risks & Mitigations

- **Service availability**: Embedding service outage could block ingestion. Mitigate with retries, circuit breaker, and lexical fallback; alert on sustained failures.
- **Storage bloat**: 768-D doubles vector storage vs 384-D; monitor table growth and consider matryoshka truncation if needed.
- **Prefix misuse**: Missing `search_query:` / `search_document:` degrades quality; enforce via code-level helper and add unit tests.
- **Patch stripping regressions**: If patch detection removes relevant conversational content, search quality drops; maintain heuristics and tests over sample emails.
- **Latency on CPU**: TEI CPU image may struggle under load; provide GPU path and consider caching hot embeddings.
- **Queue drift**: If embedding jobs stall, imports might appear complete while semantic search lags; surface queue health metrics and expose manual recovery APIs.
- **Operational misuse**: Dropping embeddings/indexes on prod during peak hours could spike load; document guardrails and require confirmation tokens in admin UI.

## Open Questions

- Should we store multiple embedding dimensionalities (e.g., 768 + 256) for tiered search, or truncate on-the-fly?
- What telemetry do we need to decide if/when to enable reranking?
- Do we introduce priority lanes for embedding jobs sourced from user-triggered rebuilds vs automated post-import refreshes?
- How do we authenticate UI-triggered destructive actions (drop all embeddings) to avoid accidental clicks? Two-step confirmation or admin PIN?

---

Archive this plan once the overhaul ships and note the final completion date.

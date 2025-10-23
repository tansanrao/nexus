# Implementation Plan – DB Refactor

Last updated: October 23, 2025  
Context: collapsing the API server onto a single SQLx pool and reshaping the schema so the rest of the v0.2 features (search upgrades, auth, notifications) have somewhere to live.

---

## Goals

- Drop `BulkWriteDb` and make every code path share one managed `PgPool` (design §§4.2, 13).
- Switch to reversible SQLx migrations and keep `sqlx migrate run` / `revert` happy in dev and CI (design §13).
- Extend the schema with the columns, tables, and indexes described in design §5 (emails embeddings + FTS fields, auth, notifications).
- Ship vector search on top of VectorChord so we get fast HNSW + RAC indexes without giving up pgvector compatibility.
- Keep startup strict: refuse to boot if migrations are pending or drifted.

## Non-Goals

- No embedding inference or reranking implementation yet (§4.3, §6.1).
- No notification fanout loops or SSE endpoints (§9).
- No UI/auth flow work; this is backend-only plumbing.
- No fancy production backfill beyond what we need to prove the migrations are solid.

## Checklist

1. **Take stock of today’s setup** ✅ (Oct 23, 2025)
   - [x] Find every call site of `BulkWriteDb` and note any assumptions about connection counts.
   - [x] Review existing migrations (`0000_initial.sql`, `0001_patch_metadata.sql`) against the design schema.
   - [x] Confirm our Postgres images already ship with vector extensions (swapped to VectorChord image).

2. **Single-pool runtime refactor** ✅ (Oct 23, 2025)
   - [x] Trim `api-server/src/db.rs` to expose one `NexusDb`.
   - [x] Update Rocket ignite fairings so migrations, job queue, and sync dispatcher all grab the same pool from state.
   - [x] Drop the extra database stanza from `Rocket.toml` and `ROCKET_DATABASES` docs; keep notes on tuning the unified pool size.
   - [ ] Document pool sizing guidance once we observe load with the new schema.

3. **Migration system refresh**
   - [x] Replace legacy migrations with a fresh reversible `0001_initial` (extensions + schema).
   - [x] Wire `sqlx::migrate!` to the new directory and make sure build/tests still compile (`cargo check`).
   - [ ] Add tooling notes (`README` / docs) about installing `sqlx-cli` and the expected `run` + `revert` workflow.
   - [ ] Update CI or local scripts so reverting the latest migration is part of the database check.

4. **Schema growth for search/auth/notifications**
   - Add `embedding VECTOR(384)`, `lex_ts`, `body_ts`, and the indexes from design §5/§6.2.
   - Create the user/auth tables (`users`, `local_user_credentials`, `user_profiles`, `user_refresh_tokens`, `user_thread_follows`, `notifications`, `notification_cursors`) with FK constraints and reasonable defaults.
   - Ensure `CREATE EXTENSION IF NOT EXISTS vector`, `vchord`, and `pg_trgm` are present via repeatable or earliest migration.
   - Keep down scripts honest—dropping everything created above.

5. **Backfill + hooks**
   - Write a simple SQL or Rust task to backfill `lex_ts`/`body_ts` for existing rows; leave embeddings NULL with a comment about the future job.
   - Make the import pipeline populate the new fields going forward (feature flag optional).
   - Sketch the admin trigger for `/admin/search/index/refresh` so the schema supports it when implemented.

6. **Tests and validation**
   - Update the integration test harness to use the single pool and the SQLx migrator.
   - Add a migration test that applies the latest migration then reverts it.
   - Extend fixtures to touch the new tables for when we start testing auth/notifications.

7. **Docs & rollout**
   - Tidy `docs/design.md` changelog once this lands.
   - Refresh Compose/env docs to mention the single pool and required extensions.
   - Stage dry run: boot from scratch, run migrations, smoke the API, then `sqlx migrate revert` the latest migration to confirm the rollback path.

## Success Criteria

- App compiles and runs with one `PgPool`, including background jobs.
- `sqlx migrate run` followed by `sqlx migrate revert` leaves the database clean.
- New schema pieces exist with the expected constraints and indexes.
- Rocket startup aborts when migrations aren’t applied.

## Risks

- **Pool contention**: ingest workers might starve. Measure and adjust pool size or add throttling if we notice pain.
- **Migration rewrite errors**: converting the legacy scripts could break fresh setups—always test with a brand-new database.
- **Extension availability**: if `pgvector`/`pg_trgm` are missing, startup fails. Double-check container images early.
- **Down scripts drift**: we need tests plus human review so rollbacks stay trustworthy.

## Open Questions

- Do we want Postgres `ENUM`s or stick with `CHECK` constraints for status columns?
- Should extension creation live in a repeatable migration or the very first numbered migration?
- What’s the right way to expose migration status in metrics (if at all)?

---

Update this checklist as we make progress; archive it when the refactor ships.

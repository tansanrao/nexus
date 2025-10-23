# API Server Testing Refactor Plan

## Goals
- Introduce comprehensive Rocket integration tests covering core REST endpoints.
- Provide isolated, reproducible database fixtures for handler and pipeline tests.
- Exercise background sync and threading workflows with realistic failure handling.
- Modernize the test harness for speed, reliability, and CI integration.

## Workstream Breakdown
1. **Test Rocket Builder**
   - Create a reusable utility to mount production routes with test configuration.
   - Provide helpers for blocking and async Rocket clients.
   - Hook in shared fairings (logging, CORS) behind test-friendly toggles.
2. **Route Integration Coverage**
   - Add end-to-end tests for authors, mailing lists, threads, and stats endpoints.
   - Seed deterministic fixture data and assert HTTP status, JSON payloads, headers.
   - Cover authorization/admin routes as guard layers mature.
3. **Database Fixture Strategy**
   - Adopt `sqlx::test` or equivalent helper for per-test ephemeral Postgres.
   - Run minimal migrations and ensure automatic cleanup.
   - Document reproducible local and CI workflows (`docs/testing-database.md`).
   - Auto-provision Postgres via `testcontainers` when no template URL is supplied.
4. **Background Workflow Testing**
   - Exercise sync queue, cache persistence, and threading algorithms asynchronously.
   - Inject network/IO failures to verify retry logic and metrics.
   - Share fixtures with integration suite to avoid duplication.
5. **Harness Improvements**
   - Integrate `cargo nextest` (or similar) for faster, flaky-resistant runs.
   - Define CI matrix (smoke, full, property-based) and document invocation.
   - Track coverage deltas and address warnings flagged by `cargo fix`/`clippy`.

## Tracking
- [x] Step 1 – Test Rocket builder utilities
- [x] Step 2 – Route integration coverage baseline *(health + mailing list happy-path covered)*
- [x] Step 3 – Database fixture strategy implemented *(`TestDatabase` helper + container guidance)*
- [ ] Step 4 – Background workflow tests in place
- [ ] Step 5 – Harness upgrades landed

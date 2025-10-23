# Nexus Design Document

Last updated: October 22, 2025

This document describes the system architecture, data model, runtime behavior, and operational practices for Nexus — a high‑performance knowledge base and browser for Linux kernel mailing lists.

## 1. Product Scope

- Users
  - Kernel developers/maintainers browsing discussions, patch series, RFCs.
  - Tooling authors and engineers analyzing contributor activity.
- Capabilities
  - Multi‑list browsing (enable/disable per list) with pagination and sorting.
  - Thread reconstruction via the JWZ algorithm with phantom handling.
  - Search by subject or full text; author discovery and activity stats.
  - Patch awareness: inline vs attachment, trailers, diffstat; UI folding.
  - Incremental syncing from lore.kernel.org mirrors.
- Out of scope (currently)
  - Outbound email/moderation, private archives, semantic search.

## 2. Architecture Overview

```
grokmirror (daemon/cron)  →  local git mirrors (public-inbox v2)
                                  │
                                  ▼
            Nexus API (Rocket)  ←→  PostgreSQL 18 (partitioned)
                   │
                   ▼
             React/Vite UI (nginx)
```

- Mirror: grokmirror continuously mirrors all lore.kernel.org repos (epochs) to disk.
- API: Rust + Rocket service providing REST API, sync orchestration, parsing, threading, import.
- DB: PostgreSQL 18 with LIST partitioning by `mailing_list_id`; global `authors` table.
- UI: React/TypeScript app, served by nginx; `/api` proxied to API.
- Cache: Unified, per‑mailing‑list cache (in‑memory DashMap + persisted bincode file) for fast JWZ threading.

## 3. Repository Map

- Backend API: `api-server/`
  - App lifecycle: `src/main.rs`, `src/lib.rs`
  - Routes: `src/routes/*`
  - Sync & import: `src/sync/*`
  - Threading (JWZ): `src/threading/*`
  - Migrations: `migrations/*.sql`
  - Tests/docs: `tests/*`, `docs/*`
- Frontend (primary): `frontend-new/` (nginx deployment)
- Frontend (legacy/dev): `frontend/`
- Infra: `docker-compose.yml`, `Makefile`, `config/postgresql.conf`, `.env.example`
- Mirroring: `grokmirror/*`
- Data (local): `data/` (mirrors, cache, postgres)

## 4. Backend Design

### 4.1 Runtime
- Rocket 0.5 + `rocket_db_pools` (SQLx 0.7)
- `gix` for git operations; `mailparse` for MIME; Rayon for CPU parallelism.
- CORS currently permissive; can be tightened via config.
- OpenAPI via `rocket_okapi`; docs mounted under `/api/docs/*` with spec at `/api/v1/openapi.json`.

### 4.2 Lifecycle & Startup
- `api-server/src/lib.rs::rocket()`
  - Initializes logging and `THREADING_CACHE_BASE_PATH` (`./cache` by default).
  - Attaches DB pools `nexus_db` (read/general) and `bulk_write_db` (imports).
  - Runs migrations on ignite; registers `PgPool` and `JobQueue` in managed state.
  - Spawns `SyncDispatcher` on liftoff.

### 4.3 Sync Pipeline
Modules under `src/sync`:

- Queue (`queue.rs`)
  - `sync_jobs` lifecycle: `waiting → parsing → threading → done/errored`.
  - Enqueue all enabled lists or individual slugs; cancellation and status.

- Git (`git.rs`)
  - Mirror path: `MIRROR_BASE_PATH/{slug}/git/{epoch}.git`.
  - Traverses branches; public‑inbox v2 `m` blob lookup; checkpoint‑aware commit discovery.

- Parser (`parser.rs`)
  - MIME parse; sanitized headers/body; subject normalization (`Re:`, `[PATCH …]`, etc.).
  - Extract `Message-ID`, `Date`, `From`, `To`, `Cc`, `In-Reply-To`, `References`.
  - Patch detection (inline vs attachment) and `PatchMetadata` (diff sections, diffstat, trailers, separator line).

- Import (`import/*`)
  - `BulkImporter` orchestrates chunk import (25,000 emails per chunk):
    1) Unique authors → bulk upsert via UNNEST.
    2) Emails → build columnar vectors → bulk insert (skip conflicts); load IDs in parallel.
    3) Recipients & references → bulk insert (UNNEST) with de‑dup + position ordering.
    4) Populate unified cache with email metadata + references.

- Threading (`threading/*`)
  - Unified cache `MailingListCache` (DashMap + bincode on disk: `{cache_dir}/{list_id}_unified_v1.bin`).
  - JWZ algorithm: containers for real/phantom messages; link by `References`; fallback `In-Reply-To`; root identification; depth‑first assembly; cycle‑safe.
  - Dispatcher computes SHA‑256 membership hash per thread (sorted email IDs) to detect unchanged threads and minimize writes; bulk upsert threads and memberships.

- Database helpers (`sync/database/*`)
  - Partition management per list; checkpoint save/load (`last_indexed_commit` in `mailing_list_repositories`).
  - Reset & migrations; session tuning helpers (`pg_config.rs`).

### 4.4 Admin/Control Plane
- Seed from lore manifest: fetch `manifest.js.gz`, parse repos per slug, insert idempotently; create partitions for new lists.
- Reset DB (dev/testing): drop all, re‑run migrations; emit instructions to seed.
- DB status/config endpoints: totals across tables; `SHOW` key settings.

## 5. Data Model (PostgreSQL)

Global tables:
- `mailing_lists(id, slug UNIQUE, name, enabled, sync_priority, created_at, last_synced_at, last_threaded_at)`
- `mailing_list_repositories(mailing_list_id, repo_url, repo_order, last_indexed_commit, created_at)`
- `authors(id, email UNIQUE, canonical_name, first_seen, last_seen)`
- `author_name_aliases(author_id, name, usage_count, first_seen, last_seen)`
- `author_mailing_list_activity(author_id, mailing_list_id, first_email_date, last_email_date, email_count, thread_count)`
- `sync_jobs(id, mailing_list_id, phase, priority, created_at, started_at, completed_at, error_message)`

Partitioned tables (LIST by `mailing_list_id`):
- `emails(id, mailing_list_id, message_id UNIQUE, git_commit_hash UNIQUE, author_id, subject, normalized_subject, date, in_reply_to, body, series_id, series_number, series_total, epoch, created_at, threaded_at, patch_type, is_patch_only, patch_metadata JSONB)`
- `threads(id, mailing_list_id, root_message_id UNIQUE, subject, start_date, last_date, message_count, membership_hash BYTEA)`
- `email_recipients(id, mailing_list_id, email_id, author_id, recipient_type {to,cc})`
- `email_references(mailing_list_id, email_id, referenced_message_id, position)`
- `thread_memberships(mailing_list_id, thread_id, email_id, depth)`

Indexes exist on hot columns (dates, author_id, normalized_subject, series_id, etc.). A partial index (`emails(threaded_at IS NULL)`) accelerates incremental threading.

Note: The design uses a global `authors` table (contrary to some older docs referring to `authors_{slug}` partitions).

## 6. API (v1)

Base: `/api/v1`

- Health
  - `GET /health`
- Mailing Lists
  - `GET /admin/mailing-lists`
  - `GET /admin/mailing-lists/:slug`
  - `GET /admin/mailing-lists/:slug/repositories`
  - `PATCH /admin/mailing-lists/:slug/toggle` { enabled }
  - `POST /admin/mailing-lists/seed`
- Sync
  - `POST /admin/sync/start` (all enabled)
  - `POST /admin/sync/queue` { mailingListSlugs }
  - `POST /admin/sync/cancel`
  - `GET /admin/sync/status`
- Database
  - `POST /admin/database/reset`
  - `GET /admin/database/status`
  - `GET /admin/database/config`
- Threads
  - `GET /:slug/threads?page&size&sortBy&order`
  - `GET /:slug/threads/search?q&searchType=subject|fullText&page&size&sortBy&order`
  - `GET /:slug/threads/:id`
- Emails
  - `GET /:slug/emails/:id`
- Authors
  - `GET /:slug/authors?q&page&size&sortBy&order`
  - `GET /:slug/authors/:id`
  - `GET /:slug/authors/:id/emails?page&size`
  - `GET /:slug/authors/:id/threads-started?page&size`
  - `GET /:slug/authors/:id/threads-participated?page&size`
- Stats
  - `GET /:slug/stats`

OpenAPI docs: `/api/docs/swagger/` and `/api/docs/rapidoc/` (spec: `/api/v1/openapi.json`).

## 7. Performance Characteristics

- Parsing: Rayon parallelism across cores; MIME parsing + b4‑style diff detection.
- Import: 25K chunk size; UNNEST bulk operations; minimize per‑row round trips; parallel ID lookups.
- Threading: single unified JWZ pass; membership hash prevents unnecessary updates.
- DB: Partitioning isolates datasets; tuned config in `config/postgresql.conf`; optional session tuning in `pg_config.rs`.

## 8. Testing Strategy

- Unit tests: parser, JWZ (phantoms, cycles), cache, manifest.
- Integration tests: route tests using `TestRocketBuilder`; ephemeral Postgres via `testcontainers` helper (`TestDatabase`).
- Planned: Background workflow tests (dispatcher, cache persistence), CI harness with `cargo nextest`, clippy/format gates, coverage.

## 9. Security Considerations

- Frontend: HTTP Basic Auth enforced at nginx (global except `/health`).
- API: no auth by default (dev‑friendly); CORS permissive. For public deployments, add API auth and restrict CORS.
- Sanitized error responses; avoid leaking DB internals.

Recommendations:
- Enable API authentication (JWT/service token) for admin endpoints.
- Restrict CORS origins to trusted frontends.
- Consider rate limiting for admin operations.

## 10. Observability & Operations

- Logging: single‑line request logs with latency; sync progress logs per phase.
- Health: `GET /api/v1/health`; nginx `/health` without auth.
- Metrics (future): expose Prometheus metrics for queue depth, phase timings, parse failures, DB bulk counts, cache size.

## 11. Deployment

- Docker Compose services: `postgres`, `api-server`, `frontend`.
- Volumes: bind mounts for `data/postgres`, `data/mirrors`, `data/cache` (overridable via `.env`).
- grokmirror: run as systemd user service or cron; see `grokmirror/README.md`.
- Environment:
  - `MIRROR_BASE_PATH` must match grokmirror toplevel.
  - `THREADING_CACHE_BASE_PATH` stores bincode caches.

## 12. Roadmap (selected)

- Observability: Prometheus metrics, richer admin UI status.
- Search: PostgreSQL FTS / trigram for ranked search and phrase queries.
- Robustness: concurrency controls, retry/backoff, shard parallelism.
- Security: API auth, CORS restrictions, RBAC for admin operations.
- UX: richer diff viewer, thread visualization, bookmarks/exports.

## 13. Development Guidelines

- Backend: `cargo fmt`, `cargo clippy`; prefer SQLx compile‑time queries; keep typed responders.
- Frontend: typed API layer; small composable components; contexts for shared state.
- Migrations: additive, idempotent; keep partition strategy consistent.
- Performance: batch work; avoid per‑row DB calls; adjust chunk size via config.

## 14. Open Questions

- Should API enforce auth during local dev when not behind nginx?
- Memory sizing guidelines for largest lists (documented thresholds/SLOs)?
- Default enabled lists post‑seed (none vs curated subset)?

## 15. File Pointers

- Rocket setup: `api-server/src/lib.rs`
- Sync dispatcher: `api-server/src/sync/dispatcher.rs`
- Git discovery: `api-server/src/sync/git.rs`
- Parser & patch metadata: `api-server/src/sync/parser.rs`
- Bulk importer: `api-server/src/sync/import/coordinator.rs`
- Unified cache: `api-server/src/threading/cache/mailing_list_cache.rs`
- JWZ algorithm: `api-server/src/threading/algorithm/jwz_threading.rs`
- Migrations: `api-server/migrations/0000_initial.sql`, `0001_patch_metadata.sql`
- Frontend API client: `frontend-new/src/lib/api.ts`
- nginx config: `frontend-new/nginx.conf`


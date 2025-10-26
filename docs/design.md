# Nexus Design Document — **v0.2**

**Last updated:** October 22, 2025

This document describes the system architecture, data model, runtime behavior, and operational notes for **Nexus** — a knowledge base and browser for Linux kernel mailing lists.

> **What’s new in v0.2 (high level)**
>
> * **DB refactor:** single `PgPool` (SQLx) for all operations; reversible/testable migrations; repeatable migration validation; stricter migration discipline.
> * **API docs:** OpenAPI generation refactor; **RapiDoc** only; Swagger UI removed. ([rapidocweb.com][1])
> * **Testing:** unit tests (pure logic), and **integration tests** via Docker Compose with privileged Postgres admin user and automatic DB create/seed/drop; migration up/down tests.
> * **Search 2.0:** hybrid **lexical + semantic** search backed by **Meilisearch** with user-provided Qwen3 embeddings and adjustable semantic weighting (no Postgres FTS); optional **cross‑encoder re‑ranker**; all OSS models. ([GitHub][2])
> * **Auth:** standards-based **OpenID Connect** (OIDC) plus **local username/password** accounts with JWT access/refresh tokens; default IdP **Keycloak** alongside managed local credential store. ([Keycloak][3])
> * **Profiles & notifications:** per‑user preferences; follow threads; SSE/WebSocket live notifications; Postgres‑backed pub/sub.
> * **Observability:** structured JSON logging (`tracing`), Prometheus metrics, health checks (liveness/readiness), optional OTEL. ([Crates.io][4])

---

## 1. Product Scope

* **Users**

  * Kernel developers/maintainers browsing discussions, patch series, RFCs.
  * Tooling authors analyzing contributor activity and list dynamics.
* **Capabilities (v0.2 additions in bold)**

  * Multi‑list browsing, pagination, sorting.
  * Thread reconstruction via JWZ algorithm with phantom handling.
  * Search: **lexical + semantic hybrid**; author discovery and activity stats.
  * **Re‑ranking (cross‑encoder) over top‑K candidates.**
  * Patch awareness: inline vs attachment, trailers, diffstat; UI folding.
  * Incremental syncing from lore.kernel.org mirrors.
  * **Hybrid authentication (OIDC SSO + local accounts), per‑user profiles, preferences, and thread‑follow notifications (real‑time feed).**
* **Out of scope (v0.2)**

  * Outbound email/moderation, private archives, cross‑repo code search.

---

## 2. Architecture Overview

```
grokmirror (daemon/cron)  →  local git mirrors (public-inbox v2)
                                  │
                                  ▼
            Nexus API (Rocket, Rust)  ←→  PostgreSQL 18
                   │
      ┌────────────┴────────────┐
      ▼                         ▼
Embeddings Service        Meilisearch CE
(TEI, Qwen3-Embedding)    (hybrid search)
      │                         ▲
      └──────────────►──────────┘
                   │
                   ▼
            React/Vite UI (nginx)
                   │
                   ├── OIDC Provider (Keycloak by default)
                   ├── Local Auth endpoints (/api/v1/auth/*, JWT issuer)
                   │
                   └── RapiDoc API docs (OpenAPI JSON from /api/v1/openapi.json)

Notifications (SSE/WebSocket):
  DB triggers → NOTIFY payloads → background listener → SSE/WebSocket hub → clients
```

* **Mirror:** grokmirror mirrors lore.kernel.org repos (epochs) to disk.
* **API:** Rust + Rocket service: REST API, sync orchestration, parsing, threading, search, auth, notifications. The API owns indexing pipelines, calls the embeddings service, and manages Meilisearch tasks/queries on behalf of the UI.
* **DB:** PostgreSQL 18 with LIST partitioning by `mailing_list_id`; global `authors`; maintains canonical threads/emails/authors but no longer carries search indexes. ([GitHub][2])
* **Search service:** Meilisearch Community Edition `v1.23.0` (private network, experimental vector store enabled via `/experimental-features`) maintains `threads` and `authors` indexes with user-provided Qwen3 embeddings and hybrid lexical/semantic scoring (exposed via adjustable `semanticRatio`).
* **Embeddings service:** Text Embeddings Inference serving `Qwen/Qwen3-Embedding-0.6B` over HTTP; used for both indexing and query-time embeddings.
* **UI:** React/Vite, served by nginx; `/api` proxied to API; **OIDC client**; **RapiDoc** for docs. ([authts.github.io][5])
* **Auth:** OIDC clients exchange tokens with provider; local users authenticate through Rocket endpoints issuing short-lived JWTs and refresh cookies.
* **Cache:** Unified per‑list cache (DashMap + bincode) for fast JWZ threading (unchanged).
* **Notifications:** Default **SSE** (simple, HTTP‑native) with Rocket’s `EventStream`; optional **WebSocket** via `rocket_ws` for interactive features. ([api.rocket.rs][6])

---

## 3. Repository Map (adjusted)

* Backend API: `api-server/`

  * App lifecycle: `src/main.rs`, `src/lib.rs`
  * Routes: `src/routes/*` (grouped by domain: `admin`, `threads`, `emails`, `authors`, `search`, `users`, `notifications`, `health`, `metrics`)
  * Sync & import: `src/sync/*`
  * Threading: `src/threading/*`
* **Search:** `src/search/` hosts Meilisearch client/service helpers, thread/author indexers, and sanitizers reused by routes, the sync dispatcher, and admin jobs.
  * **Auth:** `src/auth/*` (OIDC verifier, JWKS caching, local credential service, JWT issuer, refresh token store)
  * **Docs:** `src/docs/openapi.rs` (OpenAPI builder)
  * **Migrations:** `migrations/*.up.sql`, `*.down.sql` (**reversible**)
  * Tests/docs: `tests/*`, `docs/*`
* Frontend (primary): `frontend-new/`

  * **OIDC client setup**, RapiDoc page (`/docs/index.html`)
* Infra:

  * `docker-compose.yml` (dev/prod), **`docker-compose.test.yml` (integration tests)**
  * `Makefile`, `Taskfile.yml` (optional)
  * `config/postgresql.conf`, `.env.example`
* Mirroring: `grokmirror/*`
* Data (local): `data/` (mirrors, cache, postgres)

---

## 4. Backend Design

### 4.1 Runtime

* Rocket 0.5; `rocket_db_pools` + SQLx 0.7; `tracing`; `serde_json`; `tokio`.
* Git ops (`gix`), MIME (`mailparse`), Rayon parallelism.
* **Single SQLx `PgPool`** shared across all DB work (reads/writes/bulk). Pool sizing via `PgPoolOptions`. ([Docs.rs][7])
* CORS configurable.
* **OpenAPI:** still generated server‑side; **only RapiDoc** embeds the spec. ([rapidocweb.com][1])
* **Metrics:** Prometheus exporter (`metrics` + `metrics-exporter-prometheus`). ([Crates.io][4])

### 4.2 Lifecycle & Startup

* `api-server/src/lib.rs::rocket()`:

  * Initialize `tracing_subscriber` with JSON formatter; include request ID middleware.
  * Initialize **single** `PgPool` (register in managed state).
  * **Run SQLx migrations** at ignite (fail‑closed if drift or missing `down.sql` in non‑dev).
  * Mount routes; attach **Health** & **/metrics**.
  * Start a **DB NOTIFY listener** task for notifications (optional switch, see §9). ([PostgreSQL][8])

### 4.3 Sync Pipeline (unchanged mechanics; new embedding step)

* **Queue (`queue.rs`)**: same lifecycle.
* **Git (`git.rs`)**: same.
* **Parser (`parser.rs`)**: same; ensure **quote‑stripping** helpers plus patch hunk detection for semantic input.
* **Import (`import/*`)**: same bulk strategy; after import the dispatcher enqueues follow-up work instead of blocking the sync job:

  * Update hybrid search materialized fields (FTS `tsvector` refresh) either inline or via a dedicated index-maintenance job depending on operator settings.
* **Threading (JWZ)**: unchanged; membership hash for idempotency.

### 4.4 Admin/Control Plane

* Same seed/reset/status endpoints.
* Additional search maintenance APIs (all return queued job metadata so operators can track progress):
* **POST /admin/search/index/refresh** – enqueue a Meilisearch refresh for `threads` (optionally scoped to one mailing list) and rebuild the `authors` index.
* **POST /admin/search/index/reset** – drop both Meilisearch indexes and trigger a full reindex (threads + authors); runs as an `index_maintenance` job because it can be lengthy.

---

## 5. Data Model (PostgreSQL)

> **Extensions required (test & prod):**
>
> * `CREATE EXTENSION IF NOT EXISTS pg_trgm;` (legacy trigram ops for fuzzy comparisons). ([PostgreSQL][9])

**Global tables**

* `mailing_lists(id, slug UNIQUE, name, enabled, sync_priority, created_at, last_synced_at, last_threaded_at)`
* `mailing_list_repositories(mailing_list_id, repo_url, repo_order, last_indexed_commit, created_at)`
* `authors(id, email UNIQUE, canonical_name, first_seen, last_seen)`
* `author_name_aliases(author_id, name, usage_count, first_seen, last_seen)`
* `author_mailing_list_activity(author_id, mailing_list_id, first_email_date, last_email_date, email_count, thread_count)`
* `jobs(id, mailing_list_id NULL, job_type {import, index_maintenance}, status {queued, running, succeeded, failed, cancelled}, priority, payload JSONB, created_at, started_at, completed_at, error_message, last_heartbeat TIMESTAMPTZ)`

**Partitioned by `mailing_list_id` (LIST)**

* `emails(id, mailing_list_id, message_id UNIQUE, git_commit_hash UNIQUE, author_id, subject, normalized_subject, date, in_reply_to, body, series_id, series_number, series_total, epoch, created_at, threaded_at, patch_type, is_patch_only, patch_metadata JSONB, **embedding VECTOR(768)** (legacy), **lex_ts tsvector** (legacy), **body_ts tsvector** (legacy))`
* `threads(id, mailing_list_id, root_message_id UNIQUE, subject, start_date, last_date, message_count, membership_hash BYTEA)`
* `thread_embeddings(id, mailing_list_id, thread_id, embedding VECTOR(768), email_count INTEGER, aggregated_at TIMESTAMPTZ)` *(legacy aggregate table retained for backwards compatibility)*
* `email_recipients(id, mailing_list_id, email_id, author_id, recipient_type {to,cc})`
* `email_references(mailing_list_id, email_id, referenced_message_id, position)`
* `thread_memberships(mailing_list_id, thread_id, email_id, depth)`

**New user & notifications**

* `users(id, auth_provider {oidc, local, hybrid}, oidc_sub NULL, oidc_iss NULL, email UNIQUE NOT NULL, display_name, created_at, last_login_at, role {admin, user}, disabled bool DEFAULT false, token_version INT DEFAULT 0)`
* `local_user_credentials(user_id PK/FK, password_hash, password_updated_at, failed_attempts INT DEFAULT 0, locked_until TIMESTAMPTZ NULL, mfa_secret BYTEA NULL)`
* `user_profiles(user_id PK/FK, preferences JSONB DEFAULT '{}', updated_at)`
* `user_refresh_tokens(token_id UUID, user_id, hashed_token, created_at, expires_at, last_used_at, revoked_at NULL, device_fingerprint TEXT NULL, UNIQUE(token_id))`
* `user_thread_follows(user_id, mailing_list_id, thread_id, **level {watch, mute} default watch**, created_at, UNIQUE(user_id, mailing_list_id, thread_id))`
* `notifications(id, user_id, mailing_list_id, thread_id, email_id, type {new_reply}, created_at, read_at NULL)`
* `notification_cursors(user_id, last_seen_at, last_seen_notification_id)` (for efficient pagination/stream resume)

**Indexes (selected)**

* **Legacy FTS:** `CREATE INDEX emails_lex_ts_idx ON emails USING GIN(lex_ts);` and `... body_ts_idx ON emails USING GIN(body_ts);` (retained for backward compatibility; not used by the Meilisearch pipeline). ([PostgreSQL][10])
* **Legacy trigram:** `CREATE INDEX emails_subject_trgm ON emails USING GIN (subject gin_trgm_ops);` (still available for ad-hoc fuzzy lookups). ([PostgreSQL][9])
* **Legacy vectors:** `emails.embedding`/`thread_embeddings` remain in the schema but are not populated during the Meilisearch rollout; we plan to drop them once migration is complete.
* Incremental threading: partial index on `emails(threaded_at)` retained.
* Auth: `user_refresh_tokens(token_id)` unique index plus `CREATE INDEX user_refresh_tokens_user_idx ON user_refresh_tokens(user_id, expires_at DESC);` for revocation sweeps.

### 5.1 Job Queue Semantics

* Job lifecycle uses a normalized status vocabulary: `queued` (awaiting worker), `running` (claimed and active), `succeeded`, `failed`, and `cancelled`. Status transitions are enforced via database constraints and timestamp updates (`started_at`, `completed_at`, `last_heartbeat`).
* `job_type` controls execution logic:
  * `import` – full mailing list sync/import, responsible for writing raw email rows and scheduling follow-up work.
  * `index_maintenance` – handles REINDEX/DROP+CREATE sequences and other heavyweight maintenance tasks.
* All jobs carry a `payload` JSONB blob so admin APIs can describe scope (`mailingListSlug`, `startId`, `endId`, `forceReindex`). Workers validate the payload schema before execution.
* Admin status endpoints (`/admin/sync/status` et al.) expose the same structure so the frontend can render a unified queue, regardless of job type, and show per-job progress (`processed_count`, `total_count`) when workers emit heartbeats.

> **Note:** Keep the Meilisearch embedder dimensions aligned with the configured model (`threads-qwen3` currently uses 1024).

---

## 6. Search (v0.2)

### 6.1 Overview

* Meilisearch holds two private indexes: `threads` (one document per thread) and `authors` (aggregated people data).
* The API is the sole client. It shapes documents, persists them to Meilisearch, and proxies all queries so the UI never talks to Meili directly.
* Embeddings are generated through the Text Embeddings Inference sidecar running `Qwen/Qwen3-Embedding-0.6B`. Indexing stores vectors via Meili’s `userProvided` embedder (`threads-qwen3`, 1024 dims); query-time searches embed `q` the same way.
* Hybrid mode is always on. Endpoints expose a `semanticRatio` (default `0.35`) that callers can tune; the frontend now renders a “Semantic boost” slider alongside the search box.

### 6.2 Indexing pipeline

* The sync dispatcher calls `search::indexer::reindex_threads` after each successful mailing-list import. The job:
  * Sanitizes emails (quote/patch stripping) and constructs `discussion_text` per thread (root + top replies capped at ~24 k chars).
  * Gathers participants, patch flags, series metadata, timestamps, and generates a Qwen3 embedding for `normalized_subject + discussion_text`.
  * Removes stale documents for that mailing list and upserts the refreshed docs to Meilisearch in batches.
  * Ensures vector support is enabled via `PATCH /experimental-features` before applying index settings/embedders (idempotent).
* `reindex_authors` rebuilds the `authors` index (currently full refresh each time) with:
  * Canonical author metadata + aliases.
  * Per-mailing-list activity stats (`mailing_list_stats[...]`) so `/authors` can shape responses without hitting Postgres again.
* Admin endpoints enqueue the same helpers:
  * `POST /admin/search/index/refresh` ⇒ selective thread reindex (optional `mailingListSlug`) + full author refresh.
  * `POST /admin/search/index/reset` ⇒ drop both indexes, recreate settings, and invoke full thread+author rebuild.

### 6.3 Query behaviour

* `/api/v1/<slug>/threads/search`
  * Requires `q`; accepts `page`, `size`, optional `startDate`/`endDate`, and `semanticRatio` (0-1 clamp).
  * The API embeds the query, forwards filters to Meili (`mailing_list = '<slug>'`, epoch bounds), and normalizes `rankingScore` into `lexical_score` (0–1) for UI display.
* `/api/v1/<slug>/authors`
  * Optional `q`, `sortBy`, `order`. The API queries Meili and maps `mailing_list_stats[slug]` into the response shape expected by existing components.
* Both endpoints continue to support pagination metadata (`page`, `size`, `total`). UI defaults to 25 results for search and 50 for list views.

### 6.4 Operations

* Cluster configuration lives in `docker-compose.yml` (`meilisearch` service, mounted volume, API key env vars).
* `SearchService` centralises HTTP calls, settings management (searchable/filterable attributes, embedder declaration), and task polling.
* Queue jobs wrap long-running operations so Terraform/dashboards can inspect progress via the existing `/admin/sync/status` endpoints.
* **Ranking:** `ts_rank_cd` over `lex_ts` combined with trigram scoring, with recent activity as a tie-breaker.
* **Representative SQL fragment:** `WHERE lex_ts @@ plainto_tsquery('english', $q) ORDER BY ts_rank_cd(...) DESC`.

### 6.4 API

* `GET /api/v1/:slug/threads/search`

  * Query params: `q`, `limit`, `author`, `from`, `to`, `includePatches` (filter), `sort`.
  * Response includes `mode`, `results[]` with `thread`, `lexicalScore`, `semanticScore`, `combinedScore`, and `explanation` snippets.
* Author search remains lexical-only but shares pagination/filters via updated params schema.

---

## 7. API (v1) — changes

* **Docs**

  * **Remove Swagger UI**; serve **RapiDoc** at `/api/docs` and `/docs` (frontend). RapiDoc loads spec from `/api/v1/openapi.json`. ([rapidocweb.com][1])
  * OpenAPI generator refactor: central `openapi.rs` builds schemas, sets global **bearerAuth (OIDC)** security scheme, tags, and per‑route security.
* **Auth**

  * All **admin** and most data endpoints require auth; browsing public threads can remain anonymous behind nginx basic auth if desired.
  * Local auth endpoints:

    * `POST /api/v1/auth/register` – optional self-service sign-up (config gated) that creates a disabled account pending email verification.
    * `POST /api/v1/auth/login` – username/password login returning access token + refresh cookie.
    * `POST /api/v1/auth/refresh` – rotates refresh token, returns new access token.
    * `POST /api/v1/auth/logout` – revokes refresh token and clears cookie.
    * `POST /api/v1/auth/password/reset` – accepts verification token and new password.
  * OIDC endpoints:

    * `POST /api/v1/auth/oidc/callback` – exchanges auth code for access/refresh tokens; uses same signer as local login.
    * `POST /api/v1/auth/link` – links an authenticated OIDC user to an existing local account after password confirmation.
* **Notifications**

  * `GET /api/v1/users/me/notifications` (paged)
  * `GET /api/v1/users/me/notifications/stream` (**SSE**, `text/event-stream`), or `/ws` for WebSocket. ([api.rocket.rs][6])
* **Users**

  * `GET /api/v1/users/me` (profile, preferences)
  * `PATCH /api/v1/users/me/preferences`
  * `POST /api/v1/users/me/follows` `{ threadId, level }`, `DELETE` accordingly
* **Health & Metrics**

  * `GET /api/v1/health/live` (process up)
  * `GET /api/v1/health/ready` (DB connected, migrations applied, listener healthy)
  * `GET /metrics` (Prometheus text format). ([Crates.io][4])

---

## 8. Security & Authentication (OIDC + Local Accounts)

### 8.1 Providers & Account Types

* **Keycloak** remains the default OpenID Connect provider; Dex and Authelia stay viable OSS alternatives. ([Keycloak][3])
* Local accounts are first-class: credentials live in `local_user_credentials`, linked to `users` rows so deployments can authenticate even when an external IdP is unreachable.

### 8.2 Frontend Flows

* OIDC continues to use the **PKCE Authorization Code Flow** via `oidc-client-ts`, storing tokens in memory and silently renewing when access tokens expire. ([authts.github.io][5])
* Local sign-in submits `POST /api/v1/auth/login` with email + password; successful responses deliver the same JWT payload shape as OIDC logins so the UI can reuse downstream logic.
* After any login, the SPA calls `/api/v1/users/me` to hydrate profile/preferences and to confirm account status.

### 8.3 Local Credential Handling

* Passwords are hashed with **Argon2id** using per-user salts, memory cost ≥ 19 MiB, time cost ≥ 2, and parallelism tuned to hardware, aligning with OWASP recommendations. ([OWASP][27])
* The schema tracks `failed_attempts` and `locked_until` to enforce exponential back-off after repeated failures; unlocks require successful login, password reset, or admin action.
* Registration and reset flows require email verification tokens before enabling a local account; unverified users cannot log in or request refresh tokens.

### 8.4 Token Issuance & Lifetimes

* Nexus issues JWT access tokens (RS256) with 15‑minute lifetimes for both OIDC-backed and local sessions, keeping the exposure window short. ([Auth0][28])
* Refresh tokens expire after 7 days and rotate on every use; reused tokens are revoked and the associated `token_version` on the user row increments, forcing global logout. ([Auth0][29])
* Refresh tokens are stored server-side only as salted hashes, meeting the requirement to protect long-lived credentials at rest. ([Auth0][28])
* Password resets, manual deactivation, or OIDC role changes also bump `token_version`, invalidating outstanding refresh tokens and SSE sessions.

### 8.5 Token Storage & Session Handling

* Access tokens stay client-side in memory and are sent via the `Authorization: Bearer` header; refresh tokens are delivered in **HttpOnly, Secure, SameSite=Lax** cookies with `Path=/api/v1/auth/refresh` to resist XSS theft. ([Descope][30])
* Clients must include an `X-CSRF-Token` header that matches a value stored in the refresh-cookie payload (double-submit) before the API will mint a new access token. ([Okta][31])
* For SSE/WebSockets, `POST /api/v1/auth/session` exchanges an access token for a short-lived HttpOnly session cookie so browsers without custom headers can subscribe safely; the cookie inherits the same CSRF and SameSite policies.
* Idle-session timeout matches refresh-token expiry; background sweeper jobs purge expired `user_refresh_tokens` by index.

### 8.6 Authorization & Account Linking

* JWTs contain `sub`, `email`, `role`, and `token_version`. The backend validates issuer/audience claims and verifies signatures through JWKS with automatic cache refresh using `openidconnect`. ([Docs.rs][15])
* When an OIDC login matches an existing local email, the accounts link by updating `auth_provider` to `hybrid` and recording the OIDC subject while retaining the local credential row.
* Role mapping: `admin` is set from provider claims or internal flags; rescinding admin privileges increments `token_version` to enforce least privilege immediately.

### 8.7 Platform Hardening

* CORS restrictions remain limited to approved frontend origins in production.
* Audit trails log successful and failed login attempts (without storing raw secrets) and surface them in admin dashboards; suspicious activity can trigger forced password resets.

---

## 9. Notifications Design

* **Source events:** on `thread_memberships` insert (a new reply), server creates `notifications` for all `user_thread_follows(level=watch)` of that thread.
* **Fanout mechanism (default):**

  * **Postgres `NOTIFY`** on channel `nexus_notifications` with payload `{user_id, notification_id}`.
  * Background task **LISTENs** and enqueues to in‑process streams; each logged‑in user’s SSE stream is fed from DB (with backfill from `notification_cursors`). ([PostgreSQL][8])
* **Scaling notes:** `LISTEN/NOTIFY` is simple and effective at moderate write rates; at very high concurrency it can become a bottleneck—monitor and consider moving to **NATS (Apache‑2.0)** or **Valkey** pub/sub if needed. ([recall.ai][16])
* **Transport:**

  * **SSE** is default (one‑way, low overhead); upgrade to **WebSocket** for richer interactions. ([MDN Web Docs][17])

---

## 10. Observability & Operations

### 10.1 Logging (structured)

* Use `tracing` + `tracing_subscriber` JSON formatter; include `request_id`, `user_id`, route, latency; log levels via env. Structured JSON to stdout works well for container platforms and log collectors. ([Medium][18])
* Avoid logging PII (e.g., full email addresses) unless necessary and mark fields.

### 10.2 Metrics

* Expose `/metrics` with **Prometheus** exporter (`metrics` + `metrics-exporter-prometheus`). Counters & histograms for request rate/latency, sync phases, parse failures, DB bulk counts, SSE clients, and queue depth. ([Crates.io][4])
* Optional **OpenTelemetry**: OTLP traces to a collector; instrument key spans (sync phases, search path, DB). ([OpenTelemetry][19])

### 10.3 Health Checks

* **Liveness** `/health/live`: returns OK if process and event loop are responsive.
* **Readiness** `/health/ready`: verifies DB connectivity, migrations up, pg listener active.
  Matches common K8s probe guidance (use a higher `failureThreshold` for liveness). ([Kubernetes][20])

---

## 11. Deployment

* **Docker Compose (dev/prod)**: `postgres:18`, `api-server`, `meilisearch`, `embeddings` (TEI), `frontend`, **Keycloak**, optional **Prometheus** + **Grafana**.
* **Docker Compose (tests)**: `docker-compose.test.yml` provides:

  * `postgres-test` with admin user; optional `pg_trgm` (no vector extensions required).
  * API tests run against ephemeral DB (see §12.2); Meilisearch is currently mocked/optional.
* **Env**

  * `DATABASE_URL` for runtime; `SQLX_OFFLINE` in CI for compile‑time query checks.
  * `OIDC_ISSUER`, `OIDC_AUDIENCE`, `OIDC_JWKS_URL`, `OIDC_ALLOWED_ALGS`.
  * `RAPIDOC_PATH` (static HTML), `OPENAPI_JSON_PATH` (`/api/v1/openapi.json`).

---

## 12. Testing Strategy (v0.2)

### 12.1 Unit Tests (pure logic)

* Parser, JWZ threading (phantoms, cycles), subject normalization, patch metadata parser, search score fusion, cache behavior.

### 12.2 Integration Tests (database & end‑to‑end)

* **Tooling:** Docker Compose orchestrates Postgres (admin user), with optional Meilisearch/TEI containers when end-to-end search tests are executed.
* **Harness flow**:

  1. **Create ephemeral DB** (random name) via admin; run **reversible SQLx migrations** (`sqlx migrate run`).

     * Use **SQLx CLI reversible migrations** (`-r` flag) with paired `.up.sql` / `.down.sql`. ([Docs.rs][21])
  2. Seed minimal lists/emails for scenarios.
  3. Run tests (Rust integration tests under `tests/`), pointing to that DB.
  4. Validate **down** migrations by applying latest migration and **reverting** it in a dedicated test run to ensure rollback health.
  5. **Drop DB** at the end (even on failure via `Drop` guard / shell trap).
* **Alternative (future optional):** testcontainers‑rs Postgres module for per‑test containers. ([rust.testcontainers.org][22])
* **OpenAPI tests:** snapshot the generated spec; ensure `securitySchemes` and route tags/securities are correct.
* **Search tests:**

  * Assert thread/author documents emitted by the indexer contain expected fields (participants, discussion text trimming, stats).
  * Exercise Meilisearch hybrid scoring via the API (`semanticRatio` sweeps, normalization) using seeded fixtures or a mocked task client.
* **Notifications tests:** ensure `NOTIFY` → listener → SSE endpoint emits events (with backfill).

---

## 13. Database Migrations (refactor)

* **Single SQLx `PgPool`** for migrations & runtime (no separate bulk pool). ([Docs.rs][7])
  * Default Rocket config keeps `max_connections = 32`; adjust via `ROCKET_DATABASES__nexus_db__max_connections` after observing ingest pressure.
* **Reversible migrations**: create with `sqlx migrate add -r <name>` → emits `.up.sql` and `.down.sql`. Every schema change must be reversible; complex data migrations must include down logic or be split. ([Docs.rs][21])
* Run migrations at startup in production; abort on drift. Integration tests also run `sqlx migrate revert` to verify the `down.sql` side.
* **Partition management** remains additive; new list partitions via stored procedure or app helper.

> (If you prefer Rust‑coded migrations with explicit `up/down`, **Refinery** or **SeaORM Migration** are OSS alternatives; we’ll stay with SQLx CLI for simplicity and consistency.) ([GitHub][23])

---

## 14. OpenAPI & Documentation (refactor)

* **Generation:** Continue with `rocket_okapi`, but consolidate schema annotations and tags into `src/docs/openapi.rs`. Add global `BearerAuth` security and per‑route applies.
* **Serving:** Serve JSON at `/api/v1/openapi.json`.
* **UI:** **RapiDoc** web component (single static HTML file) mounted at `/api/docs` (API) and optionally proxied in frontend at `/docs`. Remove Swagger UI entirely. ([rapidocweb.com][1])

---

## 15. Frontend (React/Vite) Updates

* **OIDC** via `oidc-client-ts`: PKCE login, auto refresh, logout; handle multiple realms/clients via env. ([authts.github.io][5])
* **Search UI:** mode switch (lexical/semantic/hybrid), quick filters (date, list, author, patches), score explanations, clear fallback messaging when semantic mode unavailable.
* **Notifications:** EventSource client for `/notifications/stream`; fall back to WebSocket if needed.
* **Admin settings:** database/search panel surfaces queue-backed maintenance actions for lexical indexes and renders unified job status chips. Embedding controls are hidden until the feature returns.
* **Docs:** `/docs` route embedding RapiDoc.

---

## 16. Performance Characteristics

* **Parsing/Import/Threading:** unchanged.
* **Search:**

  * FTS `GIN` on `tsvector` for fast lexical; `pg_trgm` for fuzzy typos; **HNSW** vector index for semantic; hybrid fusion yields better recall/precision. ([PostgreSQL][10])
* **Embeddings:** batch inference with ONNX; tune thread pool and batch size.

---

## 17. Security Considerations

* **AuthN/AuthZ:** OIDC access tokens validated server‑side; RBAC (`admin`, `user`).
* **Session cookies:** only for SSE/WS; **HttpOnly**, **Secure**, **SameSite=Lax**; short TTL; rotation respected.
* **CORS:** locked down to known origins in prod.
* **PII:** avoid logging email addresses; mask in logs; expose only necessary fields in API.

---

## 18. Observability Notes

* **Logging:** JSON to stdout; include request IDs; redact PII. ([Medium][18])
* **Metrics:** Prometheus scrape `/metrics`; alerting:

  * Import backlog age, embedding-refresh backlog age, sync failure rate, SSE client count, 95p render/search latency, DB connection utilization.
* **Tracing (optional):** OpenTelemetry OTLP to Collector → Jaeger/Tempo. ([OpenTelemetry][19])
* **Health checks:** liveness/readiness as in §10.3. ([Kubernetes][20])

---

## 19. Ops Notes

* **Rolling deploy:**

  * Step 1: run migrations.
  * Step 2: update API (handles new schema).
  * Step 3: update frontend.
  * Step 4: verify `/health/ready`, dashboards.
* **Search index maintenance:** prefer admin APIs over manual SQL—`/admin/search/index/refresh` for lightweight vacuum/reindex and `/admin/search/index/reset` for destructive rebuilds (queues `index_maintenance` jobs). Run destructive operations during low traffic and monitor queue depth.
* **Notifications:** if high‑volume `NOTIFY` becomes a bottleneck (see §9 note), switch to **NATS JetStream** or Valkey pub/sub for fanout. ([NATS.io][24])

---

## 20. Development Guidelines

* **Backend:** `cargo fmt/clippy`; SQLx compile‑time queries (enable `offline` in CI).
* **Migrations:** always reversible; schema changes reviewed with “down” diff.
* **Testing:** `cargo nextest` encouraged (faster); ensure docker test profile tears down DB.
* **Frontend:** typed API client; componentized search and notification widgets.

---

## 21. Implementation Plan (high‑priority tasks)

1. **DB refactor**

   * Collapse to **one** SQLx `PgPool` (remove `bulk_write_db`).
   * Introduce reversible migrations; add `emails.embedding`, `lex_ts`, `body_ts`, user+notification tables and indexes.
2. **OpenAPI + RapiDoc**

   * Centralize builder; add global security; mount `/api/docs`; delete Swagger UI assets.
3. **Auth**

   * Backend: `openidconnect` verifier and JWKS cache; role mapping; cookie session exchange for SSE.
   * Frontend: OIDC client integration; auth guards; login/logout flows.
4. **Search 2.0**

   * Integrate Meilisearch + TEI (Qwen3) for hybrid search; build thread/author indexers, Meili settings, and admin refresh/reset jobs.
5. **Notifications**

   * DB triggers + writer to `notifications`; NOTIFY + background LISTEN; SSE endpoint + client.
6. **Testing**

   * Docker Compose test env; ephemeral DB create/seed/drop; migration up/down validation; search correctness tests.
7. **Observability**

   * JSON logs; Prometheus exporter; health endpoints; initial alerting rules.

---

## 22. Dependencies (FOSS only)

* **Rust**

  * Web: `rocket` 0.5; WebSocket: **`rocket_ws`**; SSE: `EventStream`. ([Docs.rs][25])
  * DB: `sqlx` 0.7, `rocket_db_pools`; **single `PgPool`**. ([Docs.rs][7])
  * Migrations: `sqlx-cli` with `-r` reversible scripts. ([Docs.rs][21])
  * Observability: `tracing`, `metrics` + `metrics-exporter-prometheus`, optional `opentelemetry`. ([Crates.io][4])
  * Auth: **`openidconnect`** crate. ([Docs.rs][15])
  * Search: `reqwest`-backed Meilisearch client + TEI Qwen3 embeddings. ([GitHub][2])
  * ML (optional): `ort` (ONNX Runtime) for embedding/rerank models.
* **Frontend**

  * **`oidc-client-ts`** (MIT) for OIDC. ([authts.github.io][5])
  * **RapiDoc** for API docs. ([rapidocweb.com][1])
* **Identity Providers**

  * **Keycloak** (Apache‑2.0), **Dex** (CNCF), **Authelia**. ([GitHub][26])
* **Optional Messaging (scale‑out)**

  * **NATS** (Apache‑2.0) for pub/sub; or **Valkey** (BSD) as Redis‑compatible OSS. ([NATS.io][24])

---

## 23. Open Questions

* Should hybrid search default to **rerank=true** (quality) or **false** (speed)?
* Retention policy for `notifications` (time‑based vs count‑based per user)?
* Per‑list embedding models (e.g., code/philosophy) or global model?

---

## 24. File Pointers (updated)

* Rocket setup: `api-server/src/lib.rs`
* Sync dispatcher: `api-server/src/sync/dispatcher.rs`
* Git discovery: `api-server/src/sync/git.rs`
* Parser & patch metadata: `api-server/src/sync/parser.rs`
* Bulk importer: `api-server/src/sync/import/coordinator.rs`
* Unified cache: `api-server/src/threading/cache/mailing_list_cache.rs`
* JWZ algorithm: `api-server/src/threading/algorithm/jwz_threading.rs`
* **Search:** `api-server/src/search/*`
* **Auth:** `api-server/src/auth/*`
* **OpenAPI:** `api-server/src/docs/openapi.rs`
* **Migrations:** `api-server/migrations/*.up.sql`, `*.down.sql`
* Frontend API client: `frontend-new/src/lib/api.ts`
* **OIDC client setup:** `frontend-new/src/lib/auth/oidc.ts`
* **RapiDoc page:** `frontend-new/public/docs/index.html`
* nginx config: `frontend-new/nginx.conf`

---

### Appendix A — Example snippets

**1) SQLx reversible migration**

```bash
sqlx migrate add -r add_semantic_search_columns
# creates YYYYMMDDHHMMSS_add_semantic_search_columns.up.sql and .down.sql
```

`*.up.sql` (excerpt)

```sql
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

ALTER TABLE emails ADD COLUMN embedding vector(768);
ALTER TABLE emails ADD COLUMN lex_ts tsvector;
ALTER TABLE emails ADD COLUMN body_ts tsvector;

CREATE INDEX IF NOT EXISTS emails_embedding_hnsw
  ON emails USING hnsw (embedding vector_cosine_ops);

CREATE INDEX IF NOT EXISTS emails_lex_ts_idx ON emails USING GIN (lex_ts);
CREATE INDEX IF NOT EXISTS emails_body_ts_idx ON emails USING GIN (body_ts);
```

`*.down.sql` (excerpt)

```sql
DROP INDEX IF EXISTS emails_body_ts_idx;
DROP INDEX IF EXISTS emails_lex_ts_idx;
DROP INDEX IF EXISTS emails_embedding_hnsw;

ALTER TABLE emails DROP COLUMN IF EXISTS body_ts;
ALTER TABLE emails DROP COLUMN IF EXISTS lex_ts;
ALTER TABLE emails DROP COLUMN IF EXISTS embedding;
```

**2) RapiDoc static page (served by frontend or `/api/docs`)**

```html
<!doctype html>
<html>
  <head><meta charset="utf-8"><title>Nexus API</title></head>
  <body>
    <rapi-doc spec-url="/api/v1/openapi.json" render-style="read" show-header="true"></rapi-doc>
    <script type="module" src="https://unpkg.com/rapidoc/dist/rapidoc-min.js"></script>
  </body>
</html>
```

(You can also vend the JS locally if you prefer no external fetch.) ([rapidocweb.com][1])

**3) Rocket SSE route (sketch)**

```rust
#[get("/notifications/stream")]
fn notifications_stream(user: AuthUser, hub: &State<Hub>) -> EventStream![] {
    hub.subscribe(user.id)
}
```

(Rocket provides an `Event`/`EventStream` API for SSE.) ([api.rocket.rs][6])

---

## Changelog

* **October 23, 2025** — Database refactor: unified Rocket onto a single SQLx pool, adopted VectorChord-based migrations, and added admin refresh hooks for search indexing.

---

## References (selected)

* RapiDoc (OpenAPI web component). ([rapidocweb.com][1])
* SQLx reversible migrations (`-r`), CLI docs. ([Docs.rs][21])
* SQLx `PgPool` docs; Rocket DB pools. ([Docs.rs][7])
* Postgres FTS & GIN, `pg_trgm`. ([PostgreSQL][10])
* Meilisearch hybrid search (`semanticRatio`, vector mixing). ([Meilisearch Docs][32])
* Meilisearch user-provided embeddings (`_vector` payloads). ([Meilisearch Docs][33])
* SSE & Rocket’s APIs; WebSockets via `rocket_ws`. ([MDN Web Docs][17])
* OIDC libs & providers: `openidconnect` crate; `oidc-client-ts`; Keycloak/Dex/Authelia. ([Docs.rs][15])
* Observability: Prometheus (`metrics` exporter); OpenTelemetry. ([Crates.io][4])
* LISTEN/NOTIFY basics & considerations. ([PostgreSQL][8])
* Password hashing (Argon2id) guidance. ([OWASP][27])
* Access vs refresh token lifetimes. ([Auth0][28])
* Refresh token rotation. ([Auth0][29])
* Secure storage of refresh tokens in cookies. ([Descope][30])
* Double-submit cookie CSRF protection. ([Okta][31])
[32]: https://www.meilisearch.com/docs/learn/advanced/hybrid_search
[33]: https://www.meilisearch.com/docs/learn/advanced/vector_databases/user_provided
[VectorChord]: https://vectorchord.ai/docs/vchord-postgres

# Nexus Design Document — **v0.2**

**Last updated:** October 22, 2025

This document describes the system architecture, data model, runtime behavior, and operational notes for **Nexus** — a knowledge base and browser for Linux kernel mailing lists.

> **What’s new in v0.2 (high level)**
>
> * **DB refactor:** single `PgPool` (SQLx) for all operations; reversible/testable migrations; repeatable migration validation; stricter migration discipline.
> * **API docs:** OpenAPI generation refactor; **RapiDoc** only; Swagger UI removed. ([rapidocweb.com][1])
> * **Testing:** unit tests (pure logic), and **integration tests** via Docker Compose with privileged Postgres admin user and automatic DB create/seed/drop; migration up/down tests.
> * **Search 2.0:** hybrid **lexical + semantic** search with **pgvector** HNSW + Postgres FTS; optional **cross‑encoder re‑ranker**; all OSS models. ([GitHub][2])
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
            Nexus API (Rocket, Rust)  ←→  PostgreSQL 18 (+ pgvector, pg_trgm)
                   │                               ▲
                   │                               │
                   ▼                               │
            React/Vite UI (nginx)  ←──────────────┘
                   │
                   ├── OIDC Provider (Keycloak by default)
                   ├── Local Auth endpoints (/api/v1/auth/*, JWT issuer)
                   │
                   └── RapiDoc API docs (OpenAPI JSON from /api/v1/openapi.json)

Notifications (SSE/WebSocket):
  DB triggers → NOTIFY payloads → background listener → SSE/WebSocket hub → clients
```

* **Mirror:** grokmirror mirrors lore.kernel.org repos (epochs) to disk.
* **API:** Rust + Rocket service: REST API, sync orchestration, parsing, threading, search, auth, notifications.
* **DB:** PostgreSQL 18 with LIST partitioning by `mailing_list_id`; global `authors`; **vector embeddings with `pgvector` HNSW**; FTS with `tsvector` + `GIN`; trigram similarity with `pg_trgm`. ([GitHub][2])
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
  * **Search:** `src/search/*` (FTS, vector, hybrid, reranker adapters)
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
* **Parser (`parser.rs`)**: same; ensure **quote‑stripping** helpers for semantic input.
* **Import (`import/*`)**: same bulk strategy; after import:

  * **Embedding job**: compute embedding for canonical text (normalized subject + body minus quotes/footers) and upsert into `emails.embedding`. Inference via **ONNX Runtime** in‑process (Rust crate), or an optional sidecar service; models below in §6.1.
  * Update hybrid search materialized fields (FTS `tsvector` refresh).
* **Threading (JWZ)**: unchanged; membership hash for idempotency.

### 4.4 Admin/Control Plane

* Same seed/reset/status endpoints.
* Additional: **/admin/search/index/refresh** (rebuild FTS/pgvector indexes per list or all).

---

## 5. Data Model (PostgreSQL)

> **Extensions required (test & prod):**
>
> * `CREATE EXTENSION IF NOT EXISTS vchord CASCADE;` (installs VectorChord and pulls in pgvector dependency) ([VectorChord][41])
> * `CREATE EXTENSION IF NOT EXISTS pg_trgm;` (trigram ops) ([PostgreSQL][9])

**Global tables**

* `mailing_lists(id, slug UNIQUE, name, enabled, sync_priority, created_at, last_synced_at, last_threaded_at)`
* `mailing_list_repositories(mailing_list_id, repo_url, repo_order, last_indexed_commit, created_at)`
* `authors(id, email UNIQUE, canonical_name, first_seen, last_seen)`
* `author_name_aliases(author_id, name, usage_count, first_seen, last_seen)`
* `author_mailing_list_activity(author_id, mailing_list_id, first_email_date, last_email_date, email_count, thread_count)`
* `sync_jobs(id, mailing_list_id, phase, priority, created_at, started_at, completed_at, error_message)`

**Partitioned by `mailing_list_id` (LIST)**

* `emails(id, mailing_list_id, message_id UNIQUE, git_commit_hash UNIQUE, author_id, subject, normalized_subject, date, in_reply_to, body, series_id, series_number, series_total, epoch, created_at, threaded_at, patch_type, is_patch_only, patch_metadata JSONB, **embedding VECTOR(384)**, **lex_ts tsvector**, **body_ts tsvector**)`
* `threads(id, mailing_list_id, root_message_id UNIQUE, subject, start_date, last_date, message_count, membership_hash BYTEA)`
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

* FTS: `CREATE INDEX emails_lex_ts_idx ON emails USING GIN(lex_ts);` and `... body_ts_idx ON emails USING GIN(body_ts);` (standard setup). ([PostgreSQL][10])
* Trigram (for fuzzy subject/author): `CREATE INDEX emails_subject_trgm ON emails USING GIN (subject gin_trgm_ops);` ([PostgreSQL][9])
* Vector (semantic): `CREATE INDEX emails_embedding_hnsw ON emails USING vchordrq (embedding vector_cosine_ops);` (or `vector_l2_ops` depending on model). VectorChord’s RAC index builds on `pgvector` operators and offers better speed/recall trade-offs than IVFFlat for many workloads. ([GitHub][2])
* Incremental threading: partial index on `emails(threaded_at)` retained.
* Auth: `user_refresh_tokens(token_id)` unique index plus `CREATE INDEX user_refresh_tokens_user_idx ON user_refresh_tokens(user_id, expires_at DESC);` for revocation sweeps.

> **Note:** Keep embedding dimension in sync with chosen model (default 384).

---

## 6. Search (v0.2)

### 6.1 Models (FOSS only)

* **Default embedding:** **BAAI/bge-small-en-v1.5** (384‑d, MIT) — strong quality, small footprint; publish ONNX weights alongside. ([Hugging Face][11])

  * Alternatives: `all-MiniLM-L6-v2` (Apache‑2.0, 384‑d); `nomic-embed-text-v1.5` (Apache‑2.0, resizable dims). ([Hugging Face][12])
* **Re‑ranker (optional, top‑K ≤ 200):** **cross-encoder/ms-marco-MiniLM‑L12‑v2** (Apache‑2.0); or L6‑v2 for speed. ([Hugging Face][13])

### 6.2 Indexing

* On import/update:

  * Build `lex_ts` (e.g., `to_tsvector('english', coalesce(subject,'') || ' ' || coalesce(body,''))`).
  * Maintain trigram indexes for fuzzy matching.
  * Infer **embedding** from normalized, quote‑stripped content; store in `emails.embedding`.

### 6.3 Query Plan

* **Modes:** `lexical | semantic | hybrid` (default hybrid).
* **Hybrid retrieval:**

  1. Lexical candidates via FTS (`ts_rank_cd`) + trigram fallback;
  2. Semantic candidates via `pgvector` KNN;
  3. **Reciprocal Rank Fusion (RRF)** or **weighted score** merge;
  4. Optional **cross‑encoder re‑rank** over top‑K;
  5. Result set supports **time‑decay boost** (e.g., logistic recency).

  * Postgres provides FTS building blocks; pgvector brings ANN KNN; combining both is a recommended pattern. ([Jonathan Katz][14])
* **Representative SQL fragments** (simplified):

  * KNN: `ORDER BY embedding <=> $query_vec LIMIT 200` (cosine/L2).
  * FTS: `WHERE lex_ts @@ plainto_tsquery('english', $q)` with `ORDER BY ts_rank_cd(...) DESC`.
  * Fusion: Do in SQL (CTEs) or in Rust (recommended for clarity).

### 6.4 API

* `GET /:slug/threads/search`

  * `q`, `mode=lexical|semantic|hybrid`, `rerank=true|false`, `k`, `timeBoost=0..1`, `filters` (author, date range, patch only, series).
* **Scoring fields** returned so UI can show “why” a hit matched.

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

* Expose `/metrics` with **Prometheus** exporter (`metrics` + `metrics-exporter-prometheus`). Counters & histograms for request rate/latency, sync phases, parse failures, DB bulk counts, SSE clients, queue depth, embedding latency. ([Crates.io][4])
* Optional **OpenTelemetry**: OTLP traces to a collector; instrument key spans (sync phases, search path, DB). ([OpenTelemetry][19])

### 10.3 Health Checks

* **Liveness** `/health/live`: returns OK if process and event loop are responsive.
* **Readiness** `/health/ready`: verifies DB connectivity, migrations up, pg listener active.
  Matches common K8s probe guidance (use a higher `failureThreshold` for liveness). ([Kubernetes][20])

---

## 11. Deployment

* **Docker Compose (dev/prod)**: `postgres:18` (with `pgvector`), `api-server`, `frontend`, **Keycloak**, optional **Prometheus** + **Grafana**.
* **Docker Compose (tests)**: `docker-compose.test.yml` provides:

  * `postgres-test` with admin user; `pgvector` and `pg_trgm` enabled.
  * API tests run against ephemeral DB (see §12.2).
* **Env**

  * `DATABASE_URL` for runtime; `SQLX_OFFLINE` in CI for compile‑time query checks.
  * `OIDC_ISSUER`, `OIDC_AUDIENCE`, `OIDC_JWKS_URL`, `OIDC_ALLOWED_ALGS`.
  * `RAPIDOC_PATH` (static HTML), `OPENAPI_JSON_PATH` (`/api/v1/openapi.json`).

---

## 12. Testing Strategy (v0.2)

### 12.1 Unit Tests (pure logic)

* Parser, JWZ threading (phantoms, cycles), subject normalization, patch metadata parser, search score fusion, reranker scoring (if mocked), cache behavior.

### 12.2 Integration Tests (database & end‑to‑end)

* **Tooling:** Docker Compose orchestrates Postgres with an admin user and required extensions.
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

  * Validate ANN queries return expected neighbors.
  * Hybrid fusion and reranking deterministic checks (seeded).
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
* **Search UI:** mode switch (lexical/semantic/hybrid), quick filters (date, list, author, patches), re‑rank toggle, score explanations.
* **Notifications:** EventSource client for `/notifications/stream`; fall back to WebSocket if needed.
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

  * Import backlog age, sync failure rate, SSE client count, 95p render/search latency, DB connection utilization.
* **Tracing (optional):** OpenTelemetry OTLP to Collector → Jaeger/Tempo. ([OpenTelemetry][19])
* **Health checks:** liveness/readiness as in §10.3. ([Kubernetes][20])

---

## 19. Ops Notes

* **Rolling deploy:**

  * Step 1: run migrations.
  * Step 2: update API (handles new schema).
  * Step 3: update frontend.
  * Step 4: verify `/health/ready`, dashboards.
* **Search index maintenance:** reindex per list via admin endpoint during low traffic.
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

   * Add pgvector extension, embedding job, HNSW index; FTS + trigram; hybrid fusion; optional re‑ranker.
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
  * Search: `pgvector` (DB extension), Postgres FTS, `pg_trgm`. ([GitHub][2])
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

ALTER TABLE emails ADD COLUMN embedding vector(384);
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
* `pgvector` HNSW indexing (HNSW vs IVFFlat). ([GitHub][2])
* VectorChord Postgres extension overview. ([VectorChord][41])
* Hybrid search with Postgres FTS + vectors. ([Jonathan Katz][14])
* SSE & Rocket’s APIs; WebSockets via `rocket_ws`. ([MDN Web Docs][17])
* OIDC libs & providers: `openidconnect` crate; `oidc-client-ts`; Keycloak/Dex/Authelia. ([Docs.rs][15])
* Observability: Prometheus (`metrics` exporter); OpenTelemetry. ([Crates.io][4])
* LISTEN/NOTIFY basics & considerations. ([PostgreSQL][8])
* Password hashing (Argon2id) guidance. ([OWASP][27])
* Access vs refresh token lifetimes. ([Auth0][28])
* Refresh token rotation. ([Auth0][29])
* Secure storage of refresh tokens in cookies. ([Descope][30])
* Double-submit cookie CSRF protection. ([Okta][31])
[VectorChord]: https://vectorchord.ai/docs/vchord-postgres

# Implementation Plan – Auth RBAC

## Goals
- Implement local username/password authentication that issues Nexus JWT access tokens and rotating refresh tokens.
- Enforce role-based access control (`user`, `admin`) across Rocket routes with auditable request guards.
- Provide endpoints for login, refresh, logout (single device and global), and SSE session exchange while keeping the design doc aligned.

## Non-Goals
- Building UI screens for login or admin management.
- Implementing OIDC provider integration beyond token verification.
- Adding MFA flows or password reset UX (these stay future iterations).

## Key Decisions
- **JWT keys**: Load RS256 private key from `NEXUS_JWT_PRIVATE_KEY_PATH`; expose public JWKS via `/api/v1/auth/keys` for debugging/rotation checks.
- **Refresh tokens**: Persist hashed (salt + SHA-512) refresh tokens with rotation-on-use and reuse detection; tracked per device fingerprint when provided.
- **Password hashing**: Use `argon2id` with parameters `m=19456`, `t=2`, `p=1`, leveraging `argon2` crate for compatibility with OWASP guidance.
- **RBAC**: Represent roles as an enum (`Role::{User,Admin}`) derived from the `users.role` column; add a `RequireAdmin` Rocket guard that wraps `AuthUser`.
- **Configuration**: Centralize auth-specific env parsing in `auth::config` (token TTLs, cookie names, CSRF secret, key paths).

## Work Breakdown
- [x] **Schema check**: Confirm migrations already create `users`, `local_user_credentials`, and `user_refresh_tokens`; add new migration only if extra columns (e.g., token salt) are required.
- [x] **Crate dependencies**: Add `argon2`, `jsonwebtoken`, `rand`, `time`, and `base64` (if not already) to `Cargo.toml`; enable `serde` features where needed.
- [x] **Auth module skeleton**: Introduce `src/auth/mod.rs` with submodules (`config`, `passwords`, `jwt`, `refresh_store`, `guards`, `routes`, `responses`). Wire module into `lib.rs` for Rocket mounting and state management.
- [x] **Config state**: Implement `AuthConfig` (JWT issuer/audience, access token TTL 15m, refresh TTL 7d, cookie settings, CSRF secret). Register as managed state.
- [x] **Password service**: Implement Argon2id hashing + verification, including error mapping to `AuthError`. Update test fixtures to accept pre-hashed inputs.
- [x] **JWT service**: Implement signer/verifier using `jsonwebtoken::EncodingKey` + `DecodingKey`, include `kid`, `role`, `token_version`, `permissions` claims. Provide helper to validate `token_version` against DB.
- [x] **Refresh token repository**: Create SQLx queries for insert, rotate (transactional), revoke, and sweep expired tokens. Ensure hashed token stored as `salt$hash` and reuse increments `token_version`.
- [x] **Request guards**: Add `AuthUser` (bearer token parser + DB check) and `RequireAdmin`. Update existing admin routes (`routes/admin.rs`, `routes/search.rs` if applicable) to enforce guards.
- [x] **Routes**: Implement Rocket handlers for `/login`, `/refresh`, `/logout`, `/session`, `/keys`, with OpenAPI specs, JSON responses, and cookie handling consistent with design doc.
- [x] **Error handling**: Extend shared error types for auth-specific errors (locked account, invalid credentials, token reuse) with consistent HTTP status mapping.
- [x] **OpenAPI integration**: Register new routes in `routes/mod.rs`, update `docs/openapi.rs` to include `Auth` tag, security schemes, and example responses.
- [x] **Background janitor**: Schedule periodic task (spawned at launch) to purge expired refresh tokens; expose shutdown signal hook.
### Deferred
- [ ] **Observability** *(deferred)*: Add tracing spans/fields for auth events, audit log for admin key introspection, increment metrics counters (`auth_login_success`, `auth_login_failure`).
- [ ] **Tests** *(deferred)*: Add integration coverage for login→refresh→logout, reuse detection, and admin-only endpoint access (unit coverage for hashing/JWT exists).
- [ ] **Docs** *(deferred)*: Extend operator docs (`docs/auth.md`) with configuration details for env vars and CLI provisioning workflow.

## Notes
- Password resets will be serviced by an operator shell script for now; no API work required this sprint.
- Global logout must drop active SSE/WebSocket sessions by notifying the hub to close user channels.
- We are deferring refresh-token audit log retention; no extra tables or retention policies needed yet.
- Account provisioning happens via `scripts/create-admin-user.sh` and `scripts/create-user.sh`; `/auth/signup` stays disabled until self-service is added.

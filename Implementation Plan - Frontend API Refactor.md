# Implementation Plan – Frontend API Refactor

## Goals
- Align the Next.js 16 frontend with the October 28, 2025 OpenAPI contract exposed at `/api/v1/` and `/admin/v1/`.
- Regenerate type-safe API bindings and centralize response adapters around the new `ApiResponse` + `meta.pagination` envelope.
- Rework data hooks, query keys, and domain services (lists, threads, authors, emails, stats) to respect the renamed paths and parameter shapes.
- Introduce real auth/session handling that speaks to `/auth/*` endpoints (login, refresh, logout, session introspection) and wires tokens into the existing `ky` client.
- Audit UI surfaces (thread browser, list explorer, author drill-ins, stats) so that pagination, sorting, and metadata flow correctly end-to-end.
- Restore admin/sync panels against the new `/admin/v1/*` control-plane API, trimming functionality that no longer exists (e.g., search maintenance).

## Non-Goals
- Changing backend behaviour or database schema—coordination will happen via API consumption only.
- Revisiting visual styling beyond what is required to surface the new data.
- Re-introducing archived Vite assets; all work happens inside `frontend/`.

## Current Contract Shifts
- Base path remains `/api/v1`, but public resources moved to `/lists`, `/authors`, `/auth`; the previous `{slug}/…` routing is gone.
- List endpoints now return `ApiResponse<T>` with `meta.pagination`, `meta.sort`, and optional filter metadata; request params are camelCase (`pageSize`, `listSlug`) and `sort` is an array of strings (e.g. `"last_date:desc"`).
- Author drill-ins require both `author_id` and the target list slug within the path (`/authors/{author_id}/lists/{slug}/threads-started`).
- Stats split into `/lists/stats` (aggregate) and `/lists/{slug}/stats` (per-list).
- Authentication exposes `/auth/login`, `/auth/refresh`, `/auth/logout`, `/auth/session`, `/auth/signup`, `/auth/keys` with JSON payloads returning access tokens + CSRF values.
- Admin control plane has its own spec at `/admin/v1/openapi.json`, covering mailing list management (`/lists`, `/lists/{slug}/repositories`, `/lists/{slug}/toggle`, `/lists/seed`), database operations (`/database/status|reset|config`), and job inspection (`/jobs`, `/jobs/{job_id}`). Search maintenance endpoints remain absent for now.

## Architecture Updates
- **Codegen pipeline:** use `npm run generate:api` to produce `schema.ts` from `docs/openapi-latest.json` (updated to the 2025-10-28 contract). Layer a new `api/adapters` module that normalizes `ApiResponse` into internal shapes consumed by hooks.
- **HTTP client:** extend `src/lib/api/http.ts` with JSON:API-esque adapters, automatic `Authorization` header injection, retry logic for 401 that hits `/auth/refresh`, and CSRF header propagation where required. Guard against envelope vs bare payload responses.
- **Domain services:** refactor `mailingLists.ts`, `threads.ts`, `authors.ts`, `emails.ts`, `stats.ts` to call the new endpoint paths, pass query objects (`params`, `page`, `pageSize`, `sort`) correctly, and unwrap meta for hooks. Introduce typed helpers for building `sort` query strings.
- **React Query hooks:** update `useThreadsList`, `useAuthorSearch`, `useMailingLists`, etc., to return `{ data, meta }` and expose pagination helpers consistent with the new schema. Re-compose query keys so slug filters move from the URL path to query params.
- **Auth handling:** add an `AuthProvider` that stores login state (access token, expiry, csrf) in memory + storage, coordinates refresh flows, and exposes hooks for UI consumption. Retrofit login/register pages to submit against `/auth/login`/`/auth/signup`, handle error payloads (`AuthErrorResponse`), and drive logout.
- **Admin surfaces:** point admin utilities at `/admin/v1/*`, ensuring list toggles, seeding, database status/reset, and job inspection views function. Hide only the Meilisearch maintenance controls until matching endpoints ship.
- **UI adjustments:** ensure thread browser, author panels, diff views, and stats widgets react to the new meta fields (e.g., `meta.pagination.totalItems`). Update placeholder copy to reflect real filtering capabilities once the backend surfaces them.
- **DX & testing:** refresh Storybook/fixtures (if any), update integration mocks, and expand Jest/Vitest coverage where we add new adapters. Continue to rely on `npm run build` and `npm run lint` as primary smoke tests.

## Workstream Checklist
- [x] **Contract & Tooling**
  - [x] Store `docs/openapi-20251028.json` (done) and keep `docs/openapi-latest.json` in sync (done).
  - [x] Regenerate `frontend/src/lib/api/schema.ts` via `npm run generate:api`; audit for breaking type deletions.
  - [x] Introduce `ApiResponse<T>` + `PaginationMeta` helpers in `types.ts`, migrating existing `PaginatedResponse` references.
- [x] **HTTP Layer & Auth**
  - [x] Extend `http.ts` with refresh-on-401 logic leveraging `/auth/refresh` and CSRF token propagation.
  - [x] Implement token persistence + `setTokenProvider` integration via a new `AuthProvider`.
  - [x] Wire login/signup/logout flows to `/auth/login`, `/auth/signup`, `/auth/logout`; handle `AuthErrorResponse`.
  - [x] Add user session bootstrap on app load using `/auth/session` + `/auth/keys` metadata for token TTL hints.
- [x] **Lists & Threads**
  - [x] Update mailing list service to use `/lists` + `/lists/{slug}`, returning `{ data, meta }`.
  - [x] Rewrite thread list/detail calls to `/lists/{slug}/threads` and `/lists/{slug}/threads/{thread_id}` with new params (`page`, `pageSize`, `sort`).
  - [x] Adapt UI components (`ThreadListPanel`, `ThreadBrowserPage`, `ThreadDetailView`, `ThreadDiffView`) to new pagination meta and ensure patch metadata (`patch_metadata.diff_sections`) remains compatible.
  - [ ] Revisit thread search (if backend provides a new endpoint); capture as follow-up if absent.
- [x] **Authors**
  - [x] Replace `/ {slug} /authors` usage with `/authors` + `listSlug` query param and nested `/authors/{id}/lists/{slug}/…` calls.
  - [x] Update hooks, tables, and drill-in views to respect `meta.pagination` and rename fields (`email_count` → `emailCount`, etc., depending on codegen casing).
  - [x] Ensure author statistics (first seen/last seen) surface correctly in UI cards.
- [x] **Emails & Diff Views**
  - [x] Point email detail fetches to `/lists/{slug}/emails/{email_id}`.
  - [x] Audit `ThreadDiffView` to use the enriched `patch_metadata` ranges from the new schema; add guards for missing bodies.
- [x] **Stats & Dashboard**
  - [x] Swap `/ {slug} /stats` for `/lists/{slug}/stats` and consume aggregate `/lists/stats` for overview widgets.
  - [x] Update any charts/metrics to accept `dateRangeStart`/`dateRangeEnd`.
- [x] **Admin & Feature Flags**
  - [x] Rewire admin UI modules (`mailing list` table, seed/toggle actions, database status/reset, job queue panels) to consume `/admin/v1/*`.
  - [ ] Feature-flag Meilisearch maintenance panels until `/admin/v1/search/index/*` endpoints land; document backend follow-ups.
- [x] **Documentation & Tooling**
  - [x] Refresh `docs/design.md` frontend sections to describe the new API usage, auth handling, and pagination semantics.
  - [x] Update `frontend/README.md` with new dev commands (`npm run generate:api`, auth env vars, etc.).
  - [ ] Note migration steps in the project CHANGELOG if one is introduced.
- [x] **Verification**
  - [x] Run `npm run lint`, `npm run build`, and targeted smoke tests across thread/author flows.
  - [ ] Add Vitest/Playwright coverage for auth login/logout boundary cases once implemented.

## Key Decisions & Open Questions
- **Sort format:** Confirm whether `sort` query values follow `field:direction` (e.g. `last_date:desc`) or `field,direction` encoding—spec hands back structured `SortDescriptor`, but request example is absent.
- **Admin future:** Decide whether to retire admin panels or wait for dedicated endpoints; document the path chosen so backend work can follow.
- **Token storage:** Prefer in-memory + `localStorage` hybrid to survive refresh? Evaluate security trade-offs (e.g. httpOnly cookies vs JS-accessible tokens) with backend owners.
- **Search parity:** The spec omits thread/author search endpoints—clarify if they are coming back or if UI should hide search affordances for now.
- **Notifications:** SSE/WebSocket endpoints are not documented; leave notification UI disabled until contract lands.

## Testing & Rollout
- Primary command suite: `npm run generate:api`, `npm run lint`, `npm run build`.
- Manual smoke: login/logout, thread pagination, author drill-ins, stats overview.
- Regression guard: consider adding contract tests that validate expected `ApiResponse` envelopes (e.g. via MSW mocks).
- Rollout strategy: ship behind a feature flag if backend cutover is staged; ensure compatibility toggles exist to fall back to legacy endpoints during migration week.

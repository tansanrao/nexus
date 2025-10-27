# Implementation Plan – API Client

Last updated: October 26, 2025  
Context: introduce a typed HTTP client for the Nexus API so the new Next.js 16 frontend (Tailwind CSS v4 + shadcn/ui) under `frontend/` can call the Rocket backend directly from the browser—no server-side rendering. Confirmed via Next.js devtools (`http://localhost:3000`, project root `/Users/tansanrao/work/nexus/frontend`) that the current app is active. The legacy Vite app now lives in `_archive/frontend-old/`.

---

## Goals

- Generate strongly typed request/response models from `openapi.json` and keep them in sync with the backend.
- Ship a reusable HTTP base that resolves `NEXT_PUBLIC_BACKEND_API_URL`, appends `/v1`, manages auth headers, and issues browser-side requests only.
- Wrap all API interactions in TanStack React Query so components get caching, background refresh, and mutation handling out of the box.
- Provide domain-oriented modules (admin, mailing lists, threads, authors) with typed helpers that match backend tags and fit Next.js client components.
- Document the workflow so future schema changes and environment configuration propagate with a single command.

## Non-Goals

- No attempt to backfill every legacy fetch immediately; we will migrate Next.js routes incrementally and deprecate legacy Vite utilities afterward.
- Auth/session management beyond existing token storage is out of scope—client will accept a token provider callback and reuse existing auth flows.
- We are not adding new API endpoints as part of this effort.

---

## Guiding Principles

- Treat the Next.js frontend as a client-rendered experience (Client Components + `use client` pages); every API call originates from the browser and must tolerate network variability. Rely on Next.js devtools runtime diagnostics to catch hydration/runtime regressions after API client refactors.
- Use TanStack React Query as the single fetch abstraction for reads/mutations so caching, polling, invalidation, and retries stay consistent.
- Keep the API base resolution deterministic: `NEXT_PUBLIC_BACKEND_API_URL` may be `/api` or a full origin; always append `/v1` before hitting endpoints.
- Preserve type safety end to end using generated TypeScript models and shared serialization helpers.
- Surface errors in a normalized shape (`{ status, message, details? }`) so UI components can render consistent toasts/banners.

## Snapshot Of The Spec (2025-10-26)

- 7 resource groups (`Health`, `Mailing Lists`, `Threads`, `Authors`, `Emails`, `Stats`, `Admin`); 24 operations total.
- Shared wrapper types: `DataResponse<T>` and `PaginatedResponse<T>`; admin endpoints return job metadata (`JobStatus`, `JobType`).
- Path parameters always include a mailing list `slug`; complex query parameters are bundled under `params` (`ThreadListParams`, `ThreadSearchParams`, author filters) and need structured serialization.
- Write operations are confined to admin endpoints:
  - `POST /admin/search/index/refresh|reset` (`SearchRefreshRequest`, `IndexMaintenanceRequest`)
  - `POST /admin/sync/queue` (`SyncRequest`)
  - `PATCH /admin/mailing-lists/{slug}/toggle` (`ToggleRequest`)

---

## Key Decisions

- **Type generation**: use `openapi-typescript@^7` to emit TypeScript definitions into `frontend/src/lib/api/schema.ts`. Add `npm run generate:api` and track snapshots in git.
- **Environment contract**: replace `VITE_API_URL` with `NEXT_PUBLIC_BACKEND_API_URL` across code, docs, Compose, and Make targets. Interpret values as either a relative path (e.g., `/api`) or absolute origin (`http://localhost:8000/api`). Provide `resolveBackendBase()` that normalizes trailing slashes and appends `/v1`, falling back to `/api` when unset.
- **HTTP transport**: continue using `ky` for concise fetch syntax, automatic JSON parsing, and abort support. Wrap it so every request uses the resolved base URL and shared error normalization.
- **React Query structure**: centralize `QueryClient` setup (provider + default options) within Next.js (e.g., `src/providers/QueryProvider.tsx`) and share query keys via `src/lib/api/queryKeys.ts`. Reads use `useQuery` with sensible stale times; mutations use `useMutation` and invalidate affected keys. Enable `@tanstack/react-query-devtools` in development and document how to toggle it.
- **Serialization helpers**: supply utilities for encoding structured `params` objects into query strings (e.g., `params.page`) matching Rocket expectations; reuse them in domain modules.
- **Error handling**: canonicalize backend errors (`status`, `message`, optional `code`, `details`) and surface them through hooks with ergonomic helpers for UI components.
- **Spec management**: keep a snapshot at `docs/openapi-<date>.json` (already captured for 2025-10-26) and ship `scripts/fetch-openapi.ts` to refresh from `/api/v1/openapi.json`, logging results via Next.js devtools so schema fetch failures surface quickly during local development.

---

## Task Checklist

- [ ] Update environment tooling: replace `VITE_API_URL` usage (Make targets, Docker Compose, docs, CI secrets) with `NEXT_PUBLIC_BACKEND_API_URL`, defaulting to `/api`, and document the new contract in `docs/design.md` + `frontend/README.md`.
- [ ] Build `resolveBackendBase()` utility that merges `NEXT_PUBLIC_BACKEND_API_URL` with `/v1`, handles trailing slashes, and exports typed `ApiEndpoints`.
- [ ] Add `openapi-typescript`, confirm `@tanstack/react-query` (v5), `@tanstack/react-query-devtools`, `ky`, Tailwind CSS v4, and shadcn/ui dependencies, and configure `npm run generate:api`.
- [ ] Update `Makefile` `up-frontend` target (and any related scripts) to launch the Next.js app from `frontend/` with `NEXT_PUBLIC_BACKEND_API_URL`, removing stale `frontend-new` references.
- [ ] Create `scripts/fetch-openapi.ts` to refresh `docs/openapi-latest.json` from `/api/v1/openapi.json`.
- [ ] Generate schema types into `frontend/src/lib/api/schema.ts` (lint/format).
- [ ] Implement `frontend/src/lib/api/http.ts` (ky instance + error parsing) and domain modules (`admin.ts`, `mailingLists.ts`, `threads.ts`, `authors.ts`, `emails.ts`, `stats.ts`) that call it.
- [ ] Define query keys and hooks under `frontend/src/lib/api/hooks/`, leveraging React Query for caches, pagination, and invalidation.
- [ ] Create a shared React Query provider (`frontend/src/providers/QueryProvider.tsx`) and register it via Next.js `app/layout.tsx`. Include React Query devtools gated behind `NEXT_PUBLIC_ENABLE_QUERY_DEVTOOLS`.
- [ ] Migrate high-traffic routes (mailing lists, thread list/detail) to the new hooks; leave TODO markers for remaining routes.
- [ ] Add smoke tests (Vitest) covering query serialization, base URL resolution, and error normalization.
- [ ] Document the workflow (docs/design.md, frontend README) covering codegen, env config, caching expectations, Next.js devtools usage, and migration steps.
- [ ] Run `npm run build` and `npm run lint`; capture Next.js devtools `get_errors` snapshot to verify clean runtime before merging.

---

## Open Questions / Follow-Ups

- Confirm how Rocket expects nested `params` query strings (`params.page=2` vs JSON). Adjust serialization helpers accordingly.
- Decide default React Query stale times per domain (mailing lists vs thread detail vs admin status dashboards).
- Determine whether mutations (admin actions) should optimistically update cached data or rely on invalidation only.
- Evaluate if we need polling for long-running jobs (`/admin/sync/status`) or lean on manual refresh.
- Clarify how error messages should surface in UI (toast vs inline) so hooks can standardize return types.
- Decide where to surface Next.js devtools documentation (README vs onboarding doc) so new contributors understand diagnostics workflow.

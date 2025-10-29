# Implementation Plan – Search API Redesign

> Companion spec updates live in `docs/design.md` (§6 “Search”) and the Meilisearch notes under `search-overhaul.md`.

## Goals

- Restore public search routes on top of the post-refactor `/api/v1` surface.
- Align request/response envelopes with the new `ApiResponse` meta format while exposing ranking and highlight details from Meilisearch.
- Extend query filters (date range, patch flag, participants, series) so the frontend and future automation can slice results without extra round-trips to Postgres.
- Keep admin index-maintenance flows working under `/admin/v1/search/*` and surface better status telemetry for ops.

## Current State Snapshot

- `SearchService` already wraps Meilisearch + Qwen3; it supports hybrid queries, vector uploads, and batch upserts (`api-server/src/search/service.rs`).
- Indexers keep `threads` and `authors` documents fresh after sync jobs, but HTTP routes were removed during the `/api/v1` rewrite.
- `ThreadSearchParams` still lives in `routes/params.rs`; it lacks most of the new filters and returns `Option<q>`, which currently results in empty payloads instead of a 400.
- Frontend expects `/lists/{slug}/threads/search` (legacy path) and a bare `ThreadSearchResponse`; neither follows the new envelope or spec.
- Admin jobs (`POST /admin/search/index/*`) enqueue refresh/reset work, but they must move to `/admin/v1/search/*` and emit richer progress metadata.

## Workstream Checklist

- [x] **Routes & Wiring**
  - [x] Add a dedicated `routes/search.rs` module with `GET /api/v1/lists/{slug}/threads/search`.
  - [x] Support optional cross-list queries (`GET /api/v1/search/threads`) guarded by a feature flag until merged.
  - [x] Provide `GET /api/v1/authors/search` with optional `mailingList` filter plurality.
- [x] **DTOs & Params**
  - [x] Extend `ThreadSearchParams` to capture `hasPatches`, `starterId`, `participantId`, `seriesId`, `sort`, and clamp `semanticRatio`.
  - [x] Emit a new `ThreadSearchHitDto` (thread summary, participants, first-post excerpt, scoring, highlights).
  - [x] Add `SearchMeta` block inside `ResponseMeta::extra` (query echo, applied filters, semantic ratio).
- [x] **Service Enhancements**
  - [x] Teach `build_thread_filters` to append Meilisearch filter clauses for the new params (including multi-value participants).
  - [x] Add optional global search plumbing (omit `mailing_list_id` filter when slug absent, but keep guard rails).
  - [x] Normalise highlights (`_formatted`) into safe HTML/text snippets for the UI.
- [x] **Admin Surface**
  - [x] Move refresh/reset endpoints to `/admin/v1/search/indexes/...` and update OpenAPI tags.
  - [x] Include processed doc counts and task IDs in responses so dashboards can poll progress.
- [ ] **Docs, Tests, Tooling**
  - [x] Regenerate OpenAPI after the routes land and update the frontend client/types.
  - [ ] Add integration tests covering lexical-only, hybrid, and filter combinations (use Testcontainers Meili).
  - [ ] Update `docs/design.md` and `search-overhaul.md` to match the new API contract (in progress via this design task).

## Key Decisions & Open Questions

- **Highlight format:** prefer returning HTML snippets with `<em>` markers (UI already sanitises) plus a plain-text fallback; confirm with the other maintainer.
- **Global search gating:** ship list-scoped search first; flip on global search once pagination + rate limits are proven.
- **Participant filters:** accept `participantId` (int) now; optionally add `participantEmail` later if requested.

## Dependencies & Sequencing

1. Land the API + DTO work behind feature flag(s) to avoid breaking the current UI.
2. Update frontend client/types to the new envelope before flipping the flag.
3. Refresh integration tests + docs, then archive this plan alongside the feature once deployed.

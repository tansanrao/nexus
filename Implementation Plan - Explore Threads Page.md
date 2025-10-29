# Implementation Plan - Explore Threads Page

## Goals
- Replace the placeholder explore threads landing screen with a real overview of sync-enabled mailing lists.
- Ensure each list tile links to the existing thread browser route for that slug.

## Tasks
- [x] Inspect mailing list API/types so we only surface `enabled` lists.
- [x] Build a lightweight client component that renders the list via React Query.
- [x] Wire up navigation to `/explore/threads/[slug]` when a list gets clicked.
- [x] Add empty/error states so the page still feels finished if no lists qualify or the request fails.
- [x] Smoke test via `npm run lint` and `npm run build` to ensure the page compiles cleanly.

## Open Questions
- Should we surface last sync timestamps or counts for each list? (Leaning no for now to keep things minimal.)

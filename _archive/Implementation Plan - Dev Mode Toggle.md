# Implementation Plan - Dev Mode Toggle

- [x] Add a shared dev mode context provider with a persisted toggle state.
- [x] Wrap the Next.js app tree with the new provider and surface a header toggle.
- [x] Clamp thread explorer queries in dev mode (≤10 pages, ≤50 threads per page) via React Query selectors.
- [x] Trim thread detail payloads to the first five emails when dev mode is enabled.
- [x] Update `docs/design.md` to capture the dev mode UX and data shaping rules.

**Notes**
- Default dev mode state will follow persisted local storage, falling back to `false` for first-time users.
- Filtering happens in React Query `select` handlers so server responses stay intact for tests.
- Toggle will live in the existing `SiteHeader` with a simple label for quick access during debugging.

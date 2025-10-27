# Implementation Plan – Thread Browser Port

## What We’re Doing
- Replace the single-column thread list on `app/explore/threads/[slug]` with the legacy two-column browser UI.
- Move pagination into the URL as `/app/explore/threads/:slug/:page` and add optional `:thread_id` segment for the detail pane.
- Preserve the rich thread view features from the archived frontend: collapsible email tree, diff toggle, Shiki-powered syntax highlighting, and aggregated diff view.

## Inputs & References
- Existing Next.js page: `frontend/app/app/explore/threads/[slug]/page.tsx`.
- Legacy implementation: `_archive/frontend-old/src/components/ThreadBrowserLayout.tsx` and related files (`ThreadList`, `ThreadView`, `ThreadDiffView`, `GitDiffViewer`, `EmailItem`, hooks, contexts).
- API hooks already available: `useThreadsList`, `useThreadDetail`, `useMailingLists`, `useThreadSearch`.
- Docs to lean on:
  - Next.js Dynamic Routes (App Router) – confirms optional catch-all pattern for `/slug/page/[[...threadId]]`.
  - shadcn `scroll-area` + existing dropdown menu components for scrollable panes and menus.

## Open Questions (resolved)
- **Timezone / formatting:** use a simplified formatter that derives locale-friendly date strings from the browser without the legacy timezone context.
- **Search:** defer full search; keep UI hooks minimal and stub search controls for now.
- **Shiki theming:** integrate with the existing global `ThemeProvider` (light/dark) instead of a separate Shiki theme selector.

## Task Checklist
- [x] Restructure the route folders to support `/app/explore/threads/:slug/:page/[[...threadId]]`, including redirect helpers so existing `/app/explore/threads/:slug` paths land on page 1.
- [x] Adjust breadcrumb + router logic so slug changes reset pagination (page → 1) and thread selection updates the optional segment instead of search params.
- [x] Port the ThreadBrowser shell: two-column flex layout inside the fixed main container with independent scroll (`ScrollArea` from shadcn) and responsive collapse behaviour.
- [x] Rebuild the left pane list:
  - [x] Map existing table data into card/list rows with selection state.
  - [x] Enforce 50-item page size and wire pagination buttons/keyboard shortcuts to the router.
  - [x] Stub search controls (UI only, no backend wiring yet) with TODO note for future integration.
- [x] Port the right pane views (thread + diff) with state toggle, loading/error placeholders, and Shiki-powered highlighting; ensure the right pane is empty when `thread_id` is missing.
- [x] Introduce simplified locale-based date/time helpers (no dedicated timezone context).
- [x] Bring over supporting utilities (diff parsers, Git diff viewer) and integrate Shiki highlighting with the app-wide `ThemeProvider`.
- [x] Update API hooks/types as needed for new metadata (e.g., lexical score) and confirm compatibility with backend response shape. *(No adjustments needed beyond verifying existing hooks handled the new params.)*
- [x] Refresh `docs/design.md` with the new URL contract and layout behaviour.

## Verification
- [ ] `npm run build` (per outstanding TODO) and run in dev with the Next.js + Chrome devtools checks requested earlier.
- [ ] Manual smoke test:
  - [ ] `/app/explore/threads/bpf/1` loads 50 entries, pagination updates URL, and retains slug dropdown behaviour.
  - [ ] Selecting a thread pushes `/thread_id` segment, loads detail view, and diff toggle behaves.
  - [ ] Removing the thread segment (back button) restores blank right pane without layout shift.
- [ ] Future follow-up: consider Playwright coverage for extreme thread lengths or narrow viewports once UI stabilises.

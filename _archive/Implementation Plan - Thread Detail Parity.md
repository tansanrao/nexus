# Implementation Plan – Thread Detail Parity

## What We’re Doing
- Realign the new Next.js thread detail UI with the archived React implementation’s behaviour and affordances.
- Restore advanced email tree interactions, quote rendering, and diff tooling without regressing the current routing/layout structure.
- Document the reintroduced utilities (timezone formatting, quote parsing) and ensure shared design docs stay accurate.

## Inputs & References
- Current implementation: `frontend/components/thread-browser/` (detail view, diff view, git diff viewer).
- Legacy sources: `_archive/frontend-old/src/components/ThreadView.tsx`, `EmailItem.tsx`, `EmailBody.tsx`, `ThreadDiffView.tsx`, `GitDiffViewer.tsx`, and supporting utils/contexts.
- Existing formatting helpers: `frontend/lib/locale-format.ts`, `frontend/lib/diff.ts`.
- Design canonical spec: `docs/design.md` section **Thread browser**.

## Key Decisions
- Keep the locale-first formatting introduced in the port while leaving room to add an explicit timezone toggle later.
- Reuse the new Shiki/highlight agent integration while layering legacy controls atop it (no dependency on the old Shiki context).
- Replace ad-hoc `<pre>` rendering with a ported quote parser that lives alongside the thread-browser components.

## Task Checklist
- [x] Recreate legacy header controls (expand/collapse/hide deep replies) and author interactions in `ThreadDetailView`.
- [x] Port the legacy `EmailBody` quote parsing/renderer alongside the thread-browser components and wire it into the detail view.
- [x] Migrate legacy per-message diff viewer behaviours (file grouping, stats, raw toggle, copy actions) into the new `git-diff-viewer`.
- [x] Expand `ThreadDiffView` with included patches list, detailed empty state, and configurable formatting helpers.
- [x] Refresh supporting docs/utilities (design spec, dependency list) to reflect the restored behaviour set.
- [x] Run `npm run build` to ensure the frontend compiles after changes.

## Verification
- [x] Manual smoke test: collapse/expand emails, hide deep replies toggle, author link navigation, per-file diff interactions, combined diff empty states.
- [x] Automated: `npm run build`.

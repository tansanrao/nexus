# Implementation Plan – Frontend Surface Simplification

## Goals
- Retire the marketing/landing surface from the Next.js app so the authenticated experience becomes the primary entry point.
- Promote the current `/app` experience to `/`, keeping navigation, layouts, and deep links intact.
- Preserve dedicated `/login` and `/register` entry points for auth flows without exposing marketing content.

## Non-Goals
- Rewriting application features inside the `/app` surface (threads, settings, search, etc.).
- Changing API contracts or backend auth behaviour.
- Introducing new visual design; we focus on route moves and removals.

## Proposed Route Structure
- `/` → former `/app` dashboard surface (with nested `/explore`, `/settings`, thread/author drill-ins).
- `/login` → existing login page (moved under an `(auth)` group for clarity).
- `/register` → port of existing `/signup` screen exposed at the new path and optionally redirect `/signup` → `/register`.
- Remove `/about`, `/contact`, and landing-only components (`Hero`, `Features`, etc.).

## Workstream Checklist
- [x] **Route migration:** Move `app/app` layout + pages to root route group, update dynamic routes, and ensure breadcrumbs/sidebar links point to the new base paths.
- [x] **Auth routes:** Relocate `app/login` and `app/signup` into `app/(auth)/` with updated file names and ensure form actions remain unchanged. Decide on redirect vs. alias for `/signup`.
- [x] **Cleanup landing assets:** Delete marketing-only directories (`components/blocks/landing`, `components/blocks/about`, `components/blocks/contact`, related public assets, and landing-specific utilities).
- [x] **Navigation & links:** Update sidebar/nav constants, CTA buttons, and any hardcoded `/app` URLs in components and docs. Add redirect middleware or `next.config.ts` rewrites from legacy `/app*` paths to the new locations.
- [x] **Metadata & SEO:** Trim landing-focused metadata/open-graph copy from `app/layout.tsx` so descriptions reflect the product rather than marketing claims, and remove unused favicon/og assets if redundant.
- [x] **Docs & onboarding:** Refresh `docs/design.md` and `frontend/README.md` to describe the streamlined surface, dev entrypoints, and updated route map.
- [ ] **Testing & verification:** Run `npm run lint` + `npm run build`, exercise critical flows (login/register pages, dashboard navigation, deep links like `/explore/threads/...`), and confirm browser reload behaviour for moved routes.

## Key Decisions & Notes
- **Route grouping:** Use Next.js route groups—e.g. `app/(app)/layout.tsx` + `app/(app)/page.tsx`—to keep layout organisation clear while exposing `/` publicly.
- **Redirect strategy:** Implement `next.config.ts` `redirects()` mapping `/app`, `/app/*`, `/signup` (if renamed) to new paths to avoid breaking saved bookmarks.
- **Archived content:** Since `_archive/frontend-old/` already holds historical UI, prefer outright deletion for landing components unless specific assets are still valuable.
- **Sidebar data:** `components/layouts/app-sidebar.tsx` and related hooks must swap `/app` prefixes for `/` while preserving nested route slugs.
- **Auth guard:** Verify any middleware or route protection logic does not hardcode `/app`; adjust login redirect targets accordingly.
- **Unauthenticated flow:** Leave auth stubs disabled and temporarily treat visitors as signed-in until backend wiring lands; keep the guard hook extensible for future enforcement.

## Resolved Questions
- Auth gating stays disabled for now; treat all visitors as authenticated and let them access the primary surface directly.
- Delete landing and marketing assets outright rather than archiving.

## Verification Checklist
- `npm run lint`
- `npm run build`
- Manual smoke: load `/`, `/explore`, `/settings`, `/explore/threads/lkml`, `/login`, `/register`
- Confirm legacy URLs (`/app`, `/app/explore`) 302/308 to new routes in local dev
- Update `docs/design.md` entry for frontend surface map

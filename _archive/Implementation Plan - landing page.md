## Context
- Replace the placeholder homepage with the shadcnblocks mainline template so `/` serves a marketing landing page.
- Keep existing dashboard/login/signup blocks but ensure routing matches `/app`, `/login`, and `/signup`.
- Align the imported template with our Next.js 16 + Tailwind v4 setup and shared shadcn UI primitives.

## Tasks
- [x] Inspect mainline template structure and identify assets/components needed for the landing page experience.
- [x] Clone the template repo into a temporary workspace and extract relevant routes, components, and styles.
- [x] Adapt the template code to our project conventions (App Router, Tailwind v4 CSS-first theme, shadcn UI utilities).
- [x] Move the dashboard route to `/app` without breaking existing sidebar layout/data.
- [x] Integrate the landing page under `app/page.tsx`, wiring any shared components or layout pieces.
- [x] Verify lint/build succeed (`npm run lint`, `npm run build`) and address any regressions.
- [x] Update `docs/design.md` with the new landing experience and route mapping.
- [x] Add supporting marketing pages (About, Contact) and shared navigation/footer elements.
- [x] Validate responsiveness via Next.js DevTools MCP and tighten copy.

## Decisions / Notes
- Use a temporary directory under `frontend/.tmp` for cloning to avoid polluting the repo.
- Prefer reusing existing UI primitives before adding new ones from the template; only copy what's necessary for the landing page.
- GitHub icon follows the shadcnblocks treatment; About/Contact pages reuse the landing layout stack so navigation stays consistent.

## Outcome
- `/` now ships a fully branded landing experience with hybrid search messaging.
- `/about` and `/contact` provide lightweight project context and contact options that match the marketing styling.
- Mobile and desktop navigation share the same GitHub link pattern to keep the CTA consistent.

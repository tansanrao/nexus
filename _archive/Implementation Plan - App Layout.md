# Implementation Plan - App Layout

## Goals
- Refactor the `/app/app/layout.tsx` dashboard shell to include an always-visible header and collapsible sidebar.
- Ensure the layout consumes the full viewport with no document scrolling and exposes a content slot for child routes.
- Centralize breadcrumb rendering in the layout header so individual pages can supply breadcrumb data without duplicating structure.

## Tasks
- [x] Update the dashboard layout component to compose `SidebarProvider`, `AppSidebar`, a new header region with trigger + breadcrumbs, and a scroll-free content container.
- [x] Introduce a layout context that lets child components register breadcrumb items (and optional header actions) with the layout.
- [x] Refactor `AppPageHeader` to use the new context (registering breadcrumbs/actions) instead of rendering its own header markup, keeping existing page APIs intact.
- [x] Adjust any dashboard pages that rely on old header markup spacing so that their body content fills the provided content area.
- [x] Validate behavior in the Next.js dev server using Next.js Devtools and Chrome DevTools to confirm the sidebar collapses correctly and no layout overflow occurs.
- [x] Update `docs/design.md` with the new dashboard layout structure and breadcrumb registration flow.

## Key Decisions & Notes
- Pages will keep calling `AppPageHeader` to define breadcrumbs; the component becomes a thin client that writes into layout context, so route files need minimal changes.
- The layout header will stay light: trigger, separator, breadcrumb trail, and optional action slot on the right.
- The content container will be `overflow-hidden` with `min-h-0` so nested components can opt into scrolling regions where necessary.
- Testing focus is manual verification in the running dev server (sidebar collapse + responsive layout) since no automated UI tests exist yet.

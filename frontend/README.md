# Nexus Frontend (Next.js 16)

This package hosts the actively developed UI for Nexus. It runs entirely in the browser, talking to the Rocket API via a generated client and TanStack React Query.

## Prerequisites

- Node.js 20.x (matches the Docker image)
- npm 10.x (ships with Node 20)
- Access to a running Nexus API (`make up` or `cargo run` inside `api-server/`)

## Environment

`NEXT_PUBLIC_BACKEND_API_URL` tells the client where the public API lives. Use a relative path like `/api` when running behind Docker Compose, or an absolute origin such as `http://localhost:8000/api` during local development. The client appends `/v1` automatically.

`NEXT_PUBLIC_ADMIN_API_URL` is optional; when omitted the admin HTTP client defaults to `/admin`. Override it when the control-plane API is hosted elsewhere.

Authentication is now wired to the live `/auth/*` endpoints. The app bootstraps a session on mount (refreshing tokens when needed) and persists credentials in `localStorage`. Use the `/login` form to obtain an access token before exercising protected features such as admin jobs/toggles.

When you start the dev server outside of Compose:

```bash
NEXT_PUBLIC_BACKEND_API_URL=http://localhost:8000/api npm run dev
```

## Scripts

- `npm run dev` – Start Next.js in development mode (uses port 3000 by default). Pair with Next.js devtools to inspect runtime errors (`npx next dev --turbo` is supported once dependencies are stabilized).
- `npm run build` – Generate the production bundle. This command must pass before opening a PR.
- `npm run start` – Run the production server (expects `npm run build` to have completed).
- `npm run lint` – Execute the shared ESLint config.
- `npm run generate:api` – Regenerate the public (`schema.ts`) and admin (`schema-admin.ts`) clients from the checked-in OpenAPI snapshots.

## Routes

- `/` – Nexus console shell (promoted from the old `/app` surface).
- `/explore` – Mailing list and thread exploration entry point (plus nested thread/detail routes).
- `/settings` – Stub settings panels (`/settings/general`, `/settings/database`, `/settings/search`).
- `/login` & `/register` – Auth forms retained for future wiring; `/signup` and `/app/*` redirect here.

## Docker Image

`frontend/Dockerfile` performs a multi-stage build (`node:20-alpine`) and runs `next start` in production mode. The Compose file propagates `NEXT_PUBLIC_BACKEND_API_URL` to both build-time and runtime so the generated client always targets the correct API base.

## Tooling Tips

- Keep the Next.js devtools panel open (`pnpm dlx @next-devtools/app` if running separately) to watch for hydration errors, API failures, and to inspect React Query cache state once the client lands.
- Run `npm run build` and `npm run lint` before pushing changes; add Vitest or Playwright coverage under `src/__tests__/` when introducing new behaviour.

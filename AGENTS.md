# Repository Guidelines

## Project Structure & Module Organization
Key workspaces: `api-server/` (Rust Rocket API, `migrations/`, integration tests), `frontend/` (nginx bundle), and `frontend-new/` (active Vite + React app). Shared docs sit in `docs/`; keep `docs/design.md`—the canonical DESIGN spec—accurate before merging. Persisted data lives in `data/`, grokmirror automation in `grokmirror/`, and root-level `docker-compose.yml` plus `Makefile` drive service orchestration.

## Design & Planning Workflow
`docs/design.md` is the source of truth for Nexus behaviour. When tackling a section, spin up `Implementation Plan - <feature>.md` in the repo root with a lightweight task checklist and key decisions—keep it friendly and avoid made-up org charts or fake owners. Update both documents with every meaningful change, and archive the plan once the feature lands.

> Reminder: this is a two-person side project. Do not invent corporate structure, fictional teams, or formal owner assignments in plans or docs.

## Build, Test, and Development Commands
Use `make build`, `make up`, and `make init` for Dockerized workflows, `make down` to stop, and `make logs-<service>` for tailing. Backend iteration happens in `api-server/` with `cargo run` and `cargo test --package api-server` (Testcontainers needs Docker). `frontend-new/` is the active UI; run `npm install` once, `npm run dev` for hot reload, and `npm run build` to verify production output. Set `VITE_API_URL` when not using Compose networking.

## Coding Style & Naming Conventions
Rust code follows `rustfmt` defaults (4-space indent, snake_case modules); run `cargo fmt --all` and `cargo clippy --all-targets --all-features` pre-push. TypeScript/JSX use 2-space indentation with the repo `eslint.config.js`; run `npm run lint` in `frontend-new/`. Favor PascalCase for React components (e.g., `ThreadView.tsx`), camelCase for helpers, and descriptive SQL migration slugs such as `20241022_add_thread_index.sql`.

## Testing Guidelines
Backend integration tests live in `api-server/tests/` and rely on Testcontainers’ Postgres; name tests with descriptive snake_case (e.g., `health_endpoint_returns_ok`) and run `cargo test --all` locally and in CI. The frontend presently leans on lint/build checks, so run `npm run build` before submitting UI changes and add Playwright or Vitest coverage under `frontend-new/src/__tests__/` when introducing new behaviour.

## Commit & Pull Request Guidelines
Switch to a feature branch (`git switch -c feat/<slug>`) instead of committing to `main`. Craft small, self-contained Conventional Commits (e.g., `feat(frontend): add thread diff view`) and note design/plan updates in the body. Each PR should include a concise overview, executed commands (`cargo test`, `npm run build`, etc.), UI screenshots or GIFs when relevant, and callouts for schema or seed changes (state affected migrations and whether `make init` is needed). Link companion backend/frontend diffs so reviewers can verify end-to-end impact.

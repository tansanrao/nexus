# Implementation Plan – Meilisearch Search Refactor

> Companion spec: `search-overhaul.md`

## Goals
- Replace the Postgres FTS/pgvector search pipeline with Meilisearch-backed hybrid search for threads and authors.
- Keep Postgres as source of truth while delegating query serving to Meilisearch with locally generated Qwen3 embeddings.
- Ensure API, sync jobs, and frontend flow operate against the new search service without regressions.

## Workstream Checklist

- [x] Infra: add a Meilisearch service to `docker-compose.yml` with a shared network for the API, including configuration env vars.
- [x] Backend config: introduce Meilisearch client settings (URL, API key), reuse the TEI endpoint at `http://100.65.8.15:8080`, and surface env wiring.
- [x] Thread indexing: implement rollup builder that produces `discussion_text`, generates embeddings, and upserts docs into Meilisearch `threads` index.
- [x] Author indexing: create/update `authors` index population jobs and swap author search endpoint to query Meilisearch.
- [x] API endpoints: refactor `/threads/search` (and related admin endpoints) to target Meilisearch hybrid search with `semanticRatio`.
- [x] Sync pipeline: replace Postgres search maintenance jobs with Meilisearch bootstrapping/reset tasks.
- [x] Frontend: adapt search requests to call the updated API parameters (semantic slider, filters) and update state typing.
- [x] Docs: refresh `docs/design.md` to describe the Meilisearch architecture and remove references to Postgres search duties.

## Key Decisions & Notes

- **Indexes**: follow `search-overhaul.md` guidance — `threads` (primary key `thread_id`) and `authors` (primary key `author_id`) with `userProvided` embeddings (`threads-qwen3`, dimension 1024).
- **Embeddings**: generate via Qwen3-Embedding-0.6B served by the TEI instance at `http://100.65.8.15:8080`; batch uploads to Meili.
- **discussion_text**: reuse the existing sanitizers to strip quotes/patches, stitch together root + top replies (target cap 16k chars) to keep embedding payloads bounded.
- **Resiliency**: wrap embedding uploads in 20 s timeout + retry guardrails and record queue heartbeats between batches so long-running reindex jobs stay observable.
- **Security**: Meilisearch stays on the private Docker network; API owns the token and proxies requests.
- **Migration**: drop Postgres search-specific schema/index maintenance code once Meili parity is verified (no partial dual mode).

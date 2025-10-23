# Nexus

Linux kernel mailing list browser and knowledge base. Nexus mirrors archives from lore.kernel.org, reconstructs threads, detects patches, and provides a fast UI for exploration.

## Highlights

- Multi‑list browsing with pagination and sorting
- Accurate JWZ threading (handles missing parents/phantoms)
- Subject/full‑text search and rich author views
- Patch‑aware email display (inline/attachment, trailers, diffstat)
- Docker‑based deployment (PostgreSQL + API + UI)

## Quick Start (Docker)

```bash
make build
make up
make init    # resets DB and seeds mailing lists
```

Services
- Frontend: http://localhost (nginx, Basic Auth)
- API: http://localhost:8000/api/v1 (Swagger at /api/docs/swagger)
- Embeddings: Hugging Face Text Embeddings Inference (`nomic-ai/nomic-embed-text-v1.5`) exposed on the Compose network at `http://embeddings:8080`

Set up grokmirror separately to keep mirrors in sync. See grokmirror/README.md.

### Database Migrations (SQLx)

Migrations are managed by [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli). Install it once:

```bash
cargo install sqlx-cli --no-default-features --features native-tls,postgres
```

With the stack running, point `DATABASE_URL` at the Postgres service (defaults shown below) and apply or revert migrations as needed:

```bash
export DATABASE_URL=postgres://postgres:changeme456@localhost:5432/nexus
sqlx migrate run          # or: make migrate-run
sqlx migrate revert --target 0   # or: make migrate-revert
```

The Makefile exposes the same commands to keep CI/local workflows consistent.

## Documentation

- Design document: docs/design.md
- Grokmirror setup: grokmirror/README.md

## Contributing

Issues and PRs welcome. Please check the design doc before proposing changes to core flows (sync, threading, schema).

## License

TBD

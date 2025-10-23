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

Set up grokmirror separately to keep mirrors in sync. See grokmirror/README.md.

## Documentation

- Design document: docs/design.md
- Grokmirror setup: grokmirror/README.md

## Contributing

Issues and PRs welcome. Please check the design doc before proposing changes to core flows (sync, threading, schema).

## License

TBD


# Implementation Plan - API Refactor

## Goal
Collapse and modernize the `api-server` HTTP surface by:
- Standardizing consumer endpoints under `/api/v1` with consistent pagination, sorting, and envelope semantics.
- Moving administrative capabilities to `/admin/v1` with a redesigned jobs system.
- Removing legacy routes and bespoke payload shapes that complicate client integrations.

## High-Level Design Decisions
- **Versioned Prefixes**: All consumer endpoints live under `/api/v1`. Administrative capabilities live under `/admin/v1`. Legacy paths (e.g., `/<slug>/threads`, `/admin/sync`) are removed.
- **Envelope & Errors**: Every success response returns `{ "data": ..., "meta": { ... } }`. Errors adopt RFC-7807-style `{ "type", "title", "status", "detail", "instance" }`.
- **Pagination & Sorting**: Query params use `page` (default 1) and `pageSize` (default 25, max 100). Sorting uses multi-value `sort` (e.g., `sort=createdAt:desc`). Responses include `meta.pagination = { page, pageSize, totalPages, totalItems }` and `meta.sort` describing applied sort order.
- **Resource Layout**:
  - `/api/v1/lists` – list catalogue plus aggregate stats under `/stats`.
  - `/api/v1/authors` – global author catalogue with list-qualified sub-resources.
  - `/api/v1/lists/{slug}/threads` and `/emails` – list-scoped hierarchy endpoints.
  - `/admin/v1/jobs` – unified job queue management with typed payloads, replacing `/admin/sync`, `/admin/search/index/*`, and related database job routes.
- **Search**: Deferred; current refactor intentionally omits search functionality until a future iteration.
- **Authentication**: Existing auth guards remain but must support new prefixes. No client session flows change beyond path updates.

## Endpoint Reference (Source of Truth)

### `/api/v1/lists`
- `GET /api/v1/lists?page&pageSize&sort`
  ```json
  {
    "data": [
      { "id": "lkml", "name": "Linux Kernel Mailing List", "description": "...", "enabled": true, "syncPriority": 5, "createdAt": "2024-06-01T00:00:00Z", "lastSyncedAt": "2025-10-10T04:12:00Z" }
    ],
    "meta": {
      "pagination": { "page": 1, "pageSize": 20, "totalPages": 3, "totalItems": 45 },
      "sort": [{ "field": "name", "direction": "asc" }]
    }
  }
  ```
- `GET /api/v1/lists/stats`
  ```json
  {
    "data": { "totalLists": 45, "totalEmails": 1285345, "totalThreads": 65912, "totalAuthors": 15432 },
    "meta": {}
  }
  ```

### `/api/v1/lists/{slug}`
- `GET /api/v1/lists/{slug}`
  ```json
  {
    "data": { "id": "lkml", "name": "Linux Kernel Mailing List", "description": "...", "enabled": true, "syncPriority": 5, "createdAt": "...", "lastSyncedAt": "..." },
    "meta": {}
  }
  ```
- `GET /api/v1/lists/{slug}/stats`
  ```json
  {
    "data": { "emailCount": 1234, "threadCount": 456, "authorCount": 321, "dateRangeStart": "1996-10-01T00:00:00Z", "dateRangeEnd": "2025-10-15T12:10:00Z" },
    "meta": { "listId": "lkml" }
  }
  ```
- `GET /api/v1/lists/{slug}/threads?page&pageSize&sort`
  ```json
  {
    "data": [
      {
        "id": 90210,
        "subject": "[PATCH v2] ...",
        "startDate": "2025-10-15T08:00:00Z",
        "lastActivity": "2025-10-16T12:00:00Z",
        "messageCount": 14,
        "starter": { "id": 42, "name": "Jane Doe", "email": "jdoe@example.com" }
      }
    ],
    "meta": {
      "pagination": { "page": 2, "pageSize": 25, "totalPages": 8, "totalItems": 182 },
      "sort": [{ "field": "lastActivity", "direction": "desc" }],
      "listId": "lkml"
    }
  }
  ```
- `GET /api/v1/lists/{slug}/threads/{threadId}`
  ```json
  {
    "data": {
      "thread": { "id": 90210, "subject": "...", "startDate": "...", "lastActivity": "...", "messageCount": 14 },
      "emails": [
        { "id": 1, "subject": "...", "author": { "id": 42, "name": "Jane Doe", "email": "..." }, "date": "...", "depth": 0 }
      ]
    },
    "meta": { "listId": "lkml" }
  }
  ```
- `GET /api/v1/lists/{slug}/emails?page&pageSize&sort`
  ```json
  {
    "data": [
      { "id": 1001, "subject": "Re: ...", "author": { "id": 5, "name": "Linus Torvalds" }, "date": "2025-10-16T12:00:00Z", "threadId": 90210 }
    ],
    "meta": {
      "pagination": { "page": 1, "pageSize": 50, "totalPages": 200, "totalItems": 10000 },
      "sort": [{ "field": "date", "direction": "desc" }],
      "listId": "lkml"
    }
  }
  ```
- `GET /api/v1/lists/{slug}/emails/{emailId}`
  ```json
  {
    "data": {
      "id": 1001,
      "subject": "Re: ...",
      "body": "...",
      "date": "2025-10-16T12:00:00Z",
      "threadId": 90210,
      "author": { "id": 5, "name": "Linus Torvalds", "email": "..." }
    },
    "meta": { "listId": "lkml" }
  }
  ```

### `/api/v1/authors`
- `GET /api/v1/authors?page&pageSize&sort&filters`
  ```json
  {
    "data": [
      {
        "id": 5,
        "email": "torvalds@linux-foundation.org",
        "canonicalName": "Linus Torvalds",
        "firstSeen": "1995-01-01T00:00:00Z",
        "lastSeen": "2025-10-16T12:00:00Z",
        "mailingLists": ["lkml", "linux-mm"],
        "emailCount": 54321,
        "threadCount": 4321
      }
    ],
    "meta": {
      "pagination": { "page": 1, "pageSize": 25, "totalPages": 6, "totalItems": 150 },
      "sort": [{ "field": "activity", "direction": "desc" }]
    }
  }
  ```
- `GET /api/v1/authors/{authorId}`
  ```json
  {
    "data": {
      "id": 5,
      "email": "torvalds@linux-foundation.org",
      "canonicalName": "Linus Torvalds",
      "mailingLists": [
        { "slug": "lkml", "emailCount": 50000, "threadCount": 4000, "firstEmailDate": "...", "lastEmailDate": "..." }
      ],
      "firstSeen": "...",
      "lastSeen": "...",
      "aliases": ["Linus Torvalds", "Linus T."]
    },
    "meta": {}
  }
  ```
- `GET /api/v1/authors/{authorId}/lists/{slug}/emails?page&pageSize&sort`
  ```json
  {
    "data": [
      { "id": 1001, "subject": "...", "date": "2025-10-16T12:00:00Z", "threadId": 90210 }
    ],
    "meta": {
      "pagination": { "page": 1, "pageSize": 20, "totalPages": 3, "totalItems": 60 },
      "sort": [{ "field": "date", "direction": "desc" }],
      "listId": "lkml"
    }
  }
  ```
- `GET /api/v1/authors/{authorId}/lists/{slug}/threads-started`
- `GET /api/v1/authors/{authorId}/lists/{slug}/threads-participated`
  - Both endpoints mirror the same envelope/pagination structure as above with resource-specific payloads.

### `/admin/v1/jobs`
- `GET /admin/v1/jobs?page&pageSize&status&type`
  ```json
  {
    "data": [
      {
        "id": "job-123",
        "type": "sync",
        "status": "queued",
        "priority": 5,
        "createdAt": "2025-10-16T12:00:00Z",
        "startedAt": null,
        "completedAt": null,
        "payload": { "listSlug": "lkml" },
        "result": null
      }
    ],
    "meta": {
      "pagination": { "page": 1, "pageSize": 20, "totalPages": 1, "totalItems": 4 },
      "filters": { "status": ["queued"] }
    }
  }
  ```
- `POST /admin/v1/jobs`
  ```json
  {
    "type": "sync",
    "payload": { "listSlugs": ["lkml", "linux-mm"] },
    "priority": 5
  }
  ```
  ```json
  {
    "data": {
      "id": "job-124",
      "type": "sync",
      "status": "queued",
      "priority": 5,
      "createdAt": "2025-10-16T12:05:00Z",
      "payload": { "listSlugs": ["lkml", "linux-mm"] }
    },
    "meta": {}
  }
  ```
- `GET /admin/v1/jobs/{jobId}` – returns job snapshot using envelope structure.
- `PATCH /admin/v1/jobs/{jobId}`
  ```json
  { "action": "cancel" }
  ```
  or
  ```json
  { "priority": 3 }
  ```
- `DELETE /admin/v1/jobs/{jobId}` – remove terminal job history (if policy permits); response uses standard envelope with `data: null`.

### Shared Conventions
- Pagination params: `page` (>=1, default 1), `pageSize` (1-100, default 25).
- Sorting: `sort=<field>:<direction>` where direction ∈ `{asc, desc}`; multiple sort fields allowed by repeating `sort`.
- Meta extensions: `meta.filters` and `meta.extra` are optional JSON objects carrying applied filters or domain-specific context (e.g., job progress), keeping the envelope stable.
- Error body format:  
  ```json
  {
    "type": "https://docs.nexus/errors/not-found",
    "title": "Resource Not Found",
    "status": 404,
    "detail": "Mailing list 'foo' does not exist",
    "instance": "/api/v1/lists/foo"
  }
  ```

## Task Breakdown (Execution Order)

1. **Spec Authoring**
   - Draft updated OpenAPI schema capturing new routes, payloads, envelopes, pagination, sorting, and error components.
   - Update `docs/design.md` with rationale, resource map, and shared conventions documented above.

2. **Routing Restructure**
   - Reorganize Rocket route modules to mirror `/api/v1` and `/admin/v1` layout.
   - Remove legacy handlers and ensure mount points only expose the new routes.
   - Implement shared pagination/sorting extractor and response helper utilities.

3. **DTO & Model Updates**
   - Introduce new response structs (`ApiResponse<T>`, `ResponseMeta`, `PaginationMeta`, etc.) matching envelope contracts.
  - Align database query layers with new pagination/sorting requirements and validated sort field maps per resource.

4. **Jobs System Rewrite**
   - Design/implement new job storage schema (SQL migrations as needed) supporting typed payloads and status transitions.
   - Build Rocket handlers for list/create/get/update/delete under `/admin/v1/jobs`.
   - Replace legacy queue APIs (`enqueue_all_enabled`, `enqueue_import_job`, etc.) with new abstractions.

5. **Auth Flow Alignment**
   - Verify guard middleware recognizes new prefixes and continues to enforce `RequireAdmin` for `/admin/v1/*`.
   - Adjust CSRF/token handling for endpoints affected by method/path changes.
   - Replace RSA key management with a symmetric signing secret (`NEXUS_JWT_SECRET`, HS256) to avoid filesystem dependencies in deployments.

6. **Client & Tooling Updates**
   - Update internal scripts/automation referencing old routes (excluding frontend for now).
   - Regenerate API clients/OpenAPI artifacts consumed by tooling.

7. **Testing & Verification**
   - Refresh integration/unit tests to cover new endpoints, pagination helpers, and jobs lifecycle.
   - Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo build`, and `cargo test --all`.
   - Fix issues surfaced by formatters/linters/tests.

## Dependencies & Considerations
- Requires canonical sort field definitions for each resource (authors, lists, threads, emails) with server-side validation.
- Jobs system rewrite likely needs new SQL migrations; coordinate schema changes with deployment tooling.
- Ensure admin auth guards remain intact after route prefix changes.
- Frontend updates deferred but must be planned once backend stabilized.

## Open Questions
- Do we expose list creation/update under `/api/v1/lists` immediately or keep read-only? (Currently read-only.)
- Should `/api/v1/authors` expose additional aggregate stats beyond list-specific data?
- Should `/api/v1/lists/stats` filter out disabled lists by default or expose a query parameter?

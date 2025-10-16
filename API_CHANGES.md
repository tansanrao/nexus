# API Refactoring Changes - v1.0

## Overview

The API has been refactored to follow REST API best practices based on industry standards. This is a **breaking change** that affects all endpoints.

**Migration Required**: All clients must be updated to work with the new API.

---

## 🔴 Breaking Changes Summary

### 1. Base URL Change
- **Old**: `/api/*`
- **New**: `/api/v1/*`

### 2. Query Parameter Naming (snake_case → camelCase)
| Old Parameter | New Parameter |
|--------------|---------------|
| `limit` | `size` |
| `sort_by` | `sortBy` |
| `search_type` | `searchType` |
| `search` | `q` (for search endpoints) |
| `mailing_list_slugs` | `mailingListSlugs` |

### 3. Response Format Changes

#### List Endpoints (Non-Paginated)
**Old**:
```json
[{...}, {...}]
```

**New**:
```json
{
  "data": [{...}, {...}]
}
```

#### List Endpoints (Paginated)
**Old**:
```json
[{...}, {...}]
```

**New**:
```json
{
  "data": [{...}, {...}],
  "page": {
    "page": 1,
    "size": 50,
    "totalPages": 10,
    "totalElements": 487
  }
}
```

### 4. Error Response Format
**Old**:
```json
{
  "error": "NotFound",
  "message": "Resource not found"
}
```

**New**:
```json
{
  "status": "NOT_FOUND",
  "code": 404,
  "timestamp": "2025-10-16T14:30:00.123Z",
  "errors": [
    {
      "type": "NOT_FOUND",
      "message": "Resource not found",
      "field": "slug"  // optional
    }
  ]
}
```

### 5. Response Field Naming (snake_case → camelCase)
| Old Field | New Field |
|-----------|-----------|
| `job_ids` | `jobIds` |
| `mailing_lists_created` | `mailingListsCreated` |
| `repositories_created` | `repositoriesCreated` |
| `partitions_created` | `partitionsCreated` |
| `mailing_list_id` | `mailingListId` |
| `mailing_list_slug` | `mailingListSlug` |
| `mailing_list_name` | `mailingListName` |
| `current_job` | `currentJob` |
| `queued_jobs` | `queuedJobs` |
| `is_running` | `isRunning` |
| `total_authors` | `totalAuthors` |
| `total_emails` | `totalEmails` |
| `total_threads` | `totalThreads` |
| `total_recipients` | `totalRecipients` |
| `total_references` | `totalReferences` |
| `total_thread_memberships` | `totalThreadMemberships` |
| `date_range_start` | `dateRangeStart` |
| `date_range_end` | `dateRangeEnd` |

---

## 📋 Endpoint Changes by Category

### Mailing Lists

#### GET `/api/v1/admin/mailing-lists`
- **Response**: Now wrapped in `{ data: [...] }`

#### GET `/api/v1/admin/mailing-lists/{slug}`
- **No changes** (single resource)

#### GET `/api/v1/admin/mailing-lists/{slug}/repositories`
- **No changes** (returns array directly - admin endpoint)

#### PATCH `/api/v1/admin/mailing-lists/{slug}/toggle`
- **No changes** to structure

#### POST `/api/v1/admin/mailing-lists/seed`
- **Response fields**: `mailingListsCreated`, `repositoriesCreated`, `partitionsCreated`

---

### Threads

#### GET `/api/v1/{slug}/threads`
**Query Parameters**:
- `page` - unchanged (default: 1)
- `limit` → `size` (default: 50, max: 100)
- `sort_by` → `sortBy` (values: `startDate`, `lastDate`, `messageCount`)
- `order` - unchanged (values: `asc`, `desc`)

**Response**: Paginated format with metadata

**Example**:
```typescript
// Old
GET /api/linux-kernel/threads?page=1&limit=50&sort_by=last_date&order=desc
// Returns: Thread[]

// New
GET /api/v1/linux-kernel/threads?page=1&size=50&sortBy=lastDate&order=desc
// Returns: { data: Thread[], page: {...} }
```

#### GET `/api/v1/{slug}/threads/search`
**Query Parameters**:
- `page` - unchanged
- `limit` → `size`
- `search` → `q`
- `search_type` → `searchType` (values: `subject`, `fullText`)
- `sort_by` → `sortBy`
- `order` - unchanged

**Response**: Paginated format

**Example**:
```typescript
// Old
GET /api/linux-kernel/threads/search?search=memory&search_type=full_text

// New
GET /api/v1/linux-kernel/threads/search?q=memory&searchType=fullText
```

#### GET `/api/v1/{slug}/threads/{threadId}`
- **No changes** to structure (single resource)

---

### Emails

#### GET `/api/v1/{slug}/emails/{emailId}`
- **No changes** to structure (single resource)

---

### Authors

#### GET `/api/v1/{slug}/authors`
**Query Parameters**:
- `search` → `q`
- `page` - unchanged
- `limit` → `size`
- `sort_by` → `sortBy` (values: `canonicalName`, `email`, `emailCount`, `threadCount`, `firstEmailDate`, `lastEmailDate`)
- `order` - unchanged

**Response**: Paginated format

#### GET `/api/v1/{slug}/authors/{authorId}`
- **No changes** to structure (single resource)

#### GET `/api/v1/{slug}/authors/{authorId}/emails`
**Query Parameters**:
- `page` - unchanged
- `limit` → `size`

**Response**: Paginated format

#### GET `/api/v1/{slug}/authors/{authorId}/threads-started`
**Query Parameters**:
- `page` - unchanged
- `limit` → `size`

**Response**: Paginated format

#### GET `/api/v1/{slug}/authors/{authorId}/threads-participated`
**Query Parameters**:
- `page` - unchanged
- `limit` → `size`

**Response**: Paginated format

---

### Stats

#### GET `/api/v1/{slug}/stats`
- **No changes** to structure

---

### Admin - Sync

#### POST `/api/v1/admin/sync/start`
- **No changes** to structure

#### POST `/api/v1/admin/sync/queue`
**Request Body**:
```json
{
  "mailingListSlugs": ["linux-kernel", "netdev"]
}
```
**Response**:
```json
{
  "jobIds": [1, 2],
  "message": "Queued 2 sync job(s)"
}
```

#### GET `/api/v1/admin/sync/status`
**Response fields**: `currentJob`, `queuedJobs`, `isRunning`

#### POST `/api/v1/admin/sync/cancel`
- **No changes** to structure

---

### Admin - Database

#### POST `/api/v1/admin/database/reset`
- **No changes** to structure

#### GET `/api/v1/admin/database/status`
**Response fields**: All snake_case → camelCase (see table above)

#### GET `/api/v1/admin/database/config`
- **No changes** to structure

---

## 🔧 Migration Guide

### TypeScript/JavaScript Clients

#### 1. Update Base URL
```typescript
// Old
const API_BASE = 'http://localhost:8000/api'

// New
const API_BASE = 'http://localhost:8000/api/v1'
```

#### 2. Update Query Parameters
```typescript
// Old
const params = {
  page: 1,
  limit: 50,
  sort_by: 'last_date',
  search_type: 'full_text'
}

// New
const params = {
  page: 1,
  size: 50,
  sortBy: 'lastDate',
  searchType: 'fullText'
}
```

#### 3. Handle Paginated Responses
```typescript
// Old
const threads = await api.threads.list(mailingList, params)
// threads is Thread[]

// New
const response = await api.threads.list(mailingList, params)
const threads = response.data
const pagination = response.page
// Access: pagination.totalPages, pagination.totalElements
```

#### 4. Handle List Response Wrappers
```typescript
// Old
const mailingLists = await api.mailingLists.list()
// mailingLists is MailingList[]

// New
const response = await api.mailingLists.list()
const mailingLists = response.data
```

#### 5. Update Error Handling
```typescript
// Old
try {
  // ...
} catch (error) {
  console.error(error.error, error.message)
}

// New
try {
  // ...
} catch (error) {
  console.error(error.status, error.code)
  error.errors.forEach(err => {
    console.error(`${err.type}: ${err.message}`)
    if (err.field) console.error(`Field: ${err.field}`)
  })
}
```

### REST Client / Postman / cURL

#### Example: List Threads
```bash
# Old
curl "http://localhost:8000/api/linux-kernel/threads?page=1&limit=50&sort_by=last_date"

# New
curl "http://localhost:8000/api/v1/linux-kernel/threads?page=1&size=50&sortBy=lastDate"
```

#### Example: Search Threads
```bash
# Old
curl "http://localhost:8000/api/linux-kernel/threads/search?search=memory&search_type=full_text"

# New
curl "http://localhost:8000/api/v1/linux-kernel/threads/search?q=memory&searchType=fullText"
```

#### Example: Queue Sync
```bash
# Old
curl -X POST http://localhost:8000/api/admin/sync/queue \
  -H "Content-Type: application/json" \
  -d '{"mailing_list_slugs": ["linux-kernel"]}'

# New
curl -X POST http://localhost:8000/api/v1/admin/sync/queue \
  -H "Content-Type: application/json" \
  -d '{"mailingListSlugs": ["linux-kernel"]}'
```

---

## ✅ What Stays the Same

1. **Authentication** - No changes (if implemented)
2. **HTTP Methods** - All methods remain the same (GET, POST, PATCH, DELETE)
3. **Status Codes** - HTTP status codes unchanged
4. **Date Format** - ISO8601 UTC format maintained
5. **Resource IDs** - ID values and formats unchanged
6. **Hierarchical Routes** - Structure like `/{slug}/threads/{id}` unchanged

---

## 📚 REST Best Practices Implemented

1. ✅ **API Versioning** - All endpoints under `/api/v1`
2. ✅ **Consistent Naming** - camelCase for JSON, kebab-case for URLs
3. ✅ **Never Return Primitives** - All responses are objects
4. ✅ **Never Return Raw Arrays** - Arrays wrapped in objects
5. ✅ **Standard Pagination** - Consistent `page` and `size` parameters with metadata
6. ✅ **Standard Error Format** - Structured errors with status, code, timestamp
7. ✅ **No Abbreviations** - All field names are descriptive
8. ✅ **Search Convention** - Using `q` parameter for search queries
9. ✅ **ISO8601 Dates** - All timestamps in UTC with 'Z' suffix
10. ✅ **No Internal Details** - Sanitized error messages

---

## 🐛 Troubleshooting

### Issue: Getting 404 errors
**Solution**: Update base URL to `/api/v1`

### Issue: Pagination not working
**Solution**: Use `size` instead of `limit` parameter

### Issue: Search not returning results
**Solution**: Use `q` instead of `search` parameter

### Issue: Accessing array directly fails
**Solution**: Access via `.data` property: `response.data`

### Issue: Sort not working
**Solution**: Use camelCase: `sortBy=lastDate` not `sort_by=last_date`

### Issue: Search type not working
**Solution**: Use camelCase: `searchType=fullText` not `search_type=full_text`

---

## 📞 Support

- **Documentation**: See this file for complete API changes
- **Backend Code**: `api-server/src/routes/*.rs`
- **Response Types**: `api-server/src/models.rs`
- **Error Handling**: `api-server/src/error.rs`

---

## 📅 Version History

- **v1.0** (2025-10-16) - Initial REST API refactoring
  - Added API versioning
  - Standardized pagination
  - Implemented proper error responses
  - Converted to camelCase naming
  - Added response wrappers

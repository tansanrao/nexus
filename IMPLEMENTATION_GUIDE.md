# Linux KB Multi-Mailing List Implementation Guide

This guide provides step-by-step instructions to complete the transformation of the BPF-focused proof-of-concept into a production-ready Linux Kernel Knowledge Base with multi-mailing list support.

## Current State (What's Done ✅)

### Phase 1: Foundation & Database Schema ✅
- **README.md**: Complete rewrite with tech stack, architecture, and API documentation
- **Database Schema**: Partitioned tables by `mailing_list_id` for billions of emails
  - `mailing_lists` and `mailing_list_repositories` tables
  - All core tables partitioned: `authors`, `emails`, `threads`, etc.
  - Partition management functions: `create_mailing_list_partitions()`, `drop_mailing_list_partitions()`
  - Default mailing lists (BPF, sched-ext) with automatic partition creation
- **Models** (`src/models.rs`): All updated with `mailing_list_id` field
  - New models: `MailingList`, `MailingListRepository`, `MailingListWithRepos`
  - Updated: `Author`, `Email`, `Thread`, `ThreadMembership`, `EmailRecipient`, etc.

### Phase 2: Git Sync System ✅
- **Git Module** (`src/sync/git.rs`): Completely refactored
  - `MailingListSyncConfig`: Configuration for multi-repo sync
  - `RepoConfig`: Individual repository configuration
  - `GitManager::sync_all_repos()`: Syncs all repositories for a mailing list
  - `GitManager::get_all_email_commits()`: Aggregates commits from all repos
  - Returns `(commit_hash, path, repo_order)` tuples

### Phase 2: Partial - Importer ⚠️
- **Importer** (`src/sync/importer.rs`):
  - ✅ Struct updated to accept `mailing_list_id`
  - ⚠️ All SQL INSERT statements need updating (see below)

### Renamed Throughout Codebase ✅
- `BpfDb` → `LinuxKbDb` in all files
- `Rocket.toml`: Database config updated
- Log messages updated

---

## What Needs To Be Done 🔧

### Phase 2 Completion: Backend Core

#### Task 1: Complete Importer Refactor

**File**: `api-server/src/sync/importer.rs`

**Changes Needed**:

1. **Update `insert_authors()` method**:
```rust
async fn insert_authors(
    &self,
    tx: &mut Transaction<'_, Postgres>,
    emails: &[(String, ParsedEmail)],
) -> Result<HashMap<String, i32>, sqlx::Error> {
    // ... existing unique_authors collection code ...

    // CHANGE: Update INSERT to include mailing_list_id
    for (email, name) in chunk {
        sqlx::query(
            "INSERT INTO authors (mailing_list_id, name, email)
             VALUES ($1, $2, $3)
             ON CONFLICT (mailing_list_id, email) DO NOTHING"
        )
        .bind(self.mailing_list_id)  // ADD THIS
        .bind(name.as_str())
        .bind(email.as_str())
        .execute(&mut **tx)
        .await?;
    }

    // CHANGE: Load author IDs with mailing_list_id filter
    let rows: Vec<(i32, String)> = sqlx::query_as(
        "SELECT id, email FROM authors WHERE mailing_list_id = $1"
    )
    .bind(self.mailing_list_id)  // ADD THIS
    .fetch_all(&mut **tx)
    .await?;

    // ... rest of method ...
}
```

2. **Update `insert_emails()` method**:
```rust
async fn insert_emails(
    &self,
    tx: &mut Transaction<'_, Postgres>,
    emails: &[(String, ParsedEmail)],
    author_cache: &HashMap<String, i32>,
) -> Result<HashMap<String, i32>, sqlx::Error> {
    // ... existing code ...

    // CHANGE: Update INSERT to include mailing_list_id
    sqlx::query(
        r#"INSERT INTO emails
           (mailing_list_id, message_id, git_commit_hash, author_id,
            subject, normalized_subject, date, in_reply_to, body,
            series_id, series_number, series_total)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
           ON CONFLICT (mailing_list_id, message_id) DO NOTHING"#
    )
    .bind(self.mailing_list_id)  // ADD THIS (shift all other binds +1)
    .bind(&email.message_id)
    .bind(commit_hash)
    .bind(author_id)
    .bind(&email.subject)
    .bind(&email.normalized_subject)
    .bind(email.date)
    .bind(&email.in_reply_to)
    .bind(&email.body)
    .bind(&series_id)
    .bind(&series_num)
    .bind(&series_total)
    .execute(&mut **tx)
    .await?;

    // CHANGE: Load email IDs with mailing_list_id filter
    let rows: Vec<(i32, String)> = sqlx::query_as(
        "SELECT id, message_id FROM emails WHERE mailing_list_id = $1"
    )
    .bind(self.mailing_list_id)
    .fetch_all(&mut **tx)
    .await?;

    // ... rest of method ...
}
```

3. **Update `insert_recipients()` method**:
```rust
// CHANGE: Add mailing_list_id to INSERT
sqlx::query(
    "INSERT INTO email_recipients (mailing_list_id, email_id, author_id, recipient_type)
     VALUES ($1, $2, $3, $4)"
)
.bind(self.mailing_list_id)  // ADD THIS
.bind(email_id)
.bind(author_id)
.bind("to")  // or "cc"
.execute(&mut **tx)
.await?;
```

4. **Update `insert_references()` method**:
```rust
// CHANGE: Add mailing_list_id to INSERT
sqlx::query(
    "INSERT INTO email_references (mailing_list_id, email_id, referenced_message_id, position)
     VALUES ($1, $2, $3, $4)
     ON CONFLICT DO NOTHING"
)
.bind(self.mailing_list_id)  // ADD THIS
.bind(email_id)
.bind(ref_msg_id)
.bind(position as i32)
.execute(&mut **tx)
.await?;
```

5. **Update `build_threads()` method**:
```rust
// CHANGE: Load emails with mailing_list_id filter
let email_rows: Vec<(i32, String, String, Option<String>, DateTime<Utc>, ...)> =
    sqlx::query_as(
        r#"SELECT e.id, e.message_id, e.subject, e.in_reply_to, e.date,
                  e.series_id, e.series_number, e.series_total
           FROM emails e
           WHERE e.mailing_list_id = $1
           ORDER BY e.date"#
    )
    .bind(self.mailing_list_id)
    .fetch_all(&mut **tx)
    .await?;

// CHANGE: Load references with mailing_list_id filter
let ref_rows: Vec<(i32, String)> = sqlx::query_as(
    "SELECT email_id, referenced_message_id
     FROM email_references
     WHERE mailing_list_id = $1
     ORDER BY email_id, position"
)
.bind(self.mailing_list_id)
.fetch_all(&mut **tx)
.await?;

// CHANGE: Insert threads with mailing_list_id
sqlx::query(
    r#"INSERT INTO threads (mailing_list_id, root_message_id, subject,
                           start_date, last_date, message_count)
       VALUES ($1, $2, $3, $4, $5, $6)"#
)
.bind(self.mailing_list_id)  // ADD THIS
.bind(&thread_info.root_message_id)
.bind(&thread_info.subject)
.bind(thread_info.start_date)
.bind(thread_info.start_date)
.bind(thread_info.emails.len() as i32)
.execute(&mut **tx)
.await?;

// CHANGE: Get thread ID with mailing_list_id filter
let thread_row: (i32,) = sqlx::query_as(
    "SELECT id FROM threads WHERE mailing_list_id = $1 AND root_message_id = $2"
)
.bind(self.mailing_list_id)
.bind(&thread_info.root_message_id)
.fetch_one(&mut **tx)
.await?;

// CHANGE: Insert memberships with mailing_list_id
sqlx::query(
    "INSERT INTO thread_memberships (mailing_list_id, thread_id, email_id, depth)
     VALUES ($1, $2, $3, $4)
     ON CONFLICT DO NOTHING"
)
.bind(self.mailing_list_id)  // ADD THIS
.bind(thread_id)
.bind(email_id)
.bind(depth)
.execute(&mut **tx)
.await?;

// CHANGE: Update thread stats with mailing_list_id filter
sqlx::query(
    r#"UPDATE threads SET
        message_count = (SELECT COUNT(*) FROM thread_memberships
                        WHERE mailing_list_id = $1 AND thread_id = $2),
        start_date = (
            SELECT MIN(e.date) FROM emails e
            JOIN thread_memberships tm ON tm.email_id = e.id
            WHERE tm.mailing_list_id = $1 AND tm.thread_id = $2
        ),
        last_date = (
            SELECT MAX(e.date) FROM emails e
            JOIN thread_memberships tm ON tm.email_id = e.id
            WHERE tm.mailing_list_id = $1 AND tm.thread_id = $2
        )
    WHERE mailing_list_id = $1 AND id = $2"#
)
.bind(self.mailing_list_id)
.bind(thread_id)
.execute(&mut **tx)
.await?;
```

---

#### Task 2: Update SyncOrchestrator

**File**: `api-server/src/sync/mod.rs`

**Changes Needed**:

1. **Update imports**:
```rust
use crate::sync::git::{GitManager, MailingListSyncConfig, RepoConfig};
```

2. **Refactor `SyncOrchestrator` struct**:
```rust
pub struct SyncOrchestrator {
    git_manager: GitManager,
    pool: PgPool,
    mailing_list_id: i32,
}

impl SyncOrchestrator {
    pub fn new(git_config: MailingListSyncConfig, pool: PgPool, mailing_list_id: i32) -> Self {
        Self {
            git_manager: GitManager::new(git_config),
            pool,
            mailing_list_id,
        }
    }

    // ... methods ...
}
```

3. **Update `run_sync()` method**:
```rust
pub async fn run_sync(&self, job_manager: Arc<tokio::sync::Mutex<JobManager>>)
    -> Result<ImportStats, String> {

    log::info!("=== Starting full sync for mailing list {} ===", self.mailing_list_id);

    // Phase 1: Git Sync - CHANGE to sync_all_repos()
    {
        let mgr = job_manager.lock().await;
        if mgr.is_cancelled() {
            return Err("Sync cancelled".to_string());
        }
        mgr.update_status(JobStatus::Syncing, "Syncing git repositories".to_string()).await;
    }

    self.git_manager
        .sync_all_repos()  // CHANGED from sync_mirror()
        .map_err(|e| format!("Git sync failed: {}", e))?;

    // Phase 2: Commit Discovery - CHANGE to get_all_email_commits()
    {
        let mgr = job_manager.lock().await;
        if mgr.is_cancelled() {
            return Err("Sync cancelled".to_string());
        }
        mgr.update_phase_details("Discovering commits across all repositories...").await;
    }

    let commits = self.git_manager
        .get_all_email_commits()  // CHANGED - returns Vec<(hash, path, repo_order)>
        .map_err(|e| format!("Failed to get commits: {}", e))?;

    let total_commits = commits.len();
    log::info!("Found {} email commits across all repositories", total_commits);

    // Phase 3: Email Parsing
    let mut parsed_emails: Vec<(String, ParsedEmail)> = Vec::new();

    for (idx, (commit_hash, path, repo_order)) in commits.iter().enumerate() {
        // ... progress updates ...

        // CHANGE: Pass repo_order to get_blob_data
        let blob_data = match self.git_manager.get_blob_data(commit_hash, path, *repo_order) {
            Ok(data) => data,
            Err(e) => {
                log::warn!("Failed to get blob for commit {}: {}", commit_hash, e);
                continue;
            }
        };

        // ... parse email ...
    }

    // Phase 4: Database Import - CHANGE to pass mailing_list_id
    let importer = Importer::new(self.pool.clone(), self.mailing_list_id);
    let stats = importer
        .import_emails(parsed_emails)
        .await
        .map_err(|e| format!("Database import failed: {}", e))?;

    // ... rest of method ...
}
```

---

#### Task 3: Update Admin Routes

**File**: `api-server/src/routes/admin.rs`

**Changes Needed**:

1. **Update `start_sync` endpoint** to load mailing list config from database:
```rust
#[post("/admin/sync/<slug>/start")]
pub async fn start_sync(
    slug: String,
    pool: &State<sqlx::PgPool>,
    job_manager: &State<Arc<Mutex<JobManager>>>,
) -> Result<Json<SyncStartResponse>, ApiError> {
    // Load mailing list from database
    let list: (i32, String) = sqlx::query_as(
        "SELECT id, slug FROM mailing_lists WHERE slug = $1 AND enabled = true"
    )
    .bind(&slug)
    .fetch_one(pool.inner())
    .await
    .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;

    let (list_id, list_slug) = list;

    // Load repositories for this mailing list
    let repos: Vec<(String, i32)> = sqlx::query_as(
        "SELECT repo_url, repo_order FROM mailing_list_repositories
         WHERE mailing_list_id = $1 ORDER BY repo_order"
    )
    .bind(list_id)
    .fetch_all(pool.inner())
    .await?;

    // Build repo configs
    let repo_configs: Vec<RepoConfig> = repos
        .into_iter()
        .map(|(url, order)| RepoConfig { url, order })
        .collect();

    // Create sync config
    let git_config = MailingListSyncConfig::new(list_id, list_slug.clone(), repo_configs);

    // Start job
    let manager = job_manager.inner().lock().await;
    let job_id = manager.start_job().await
        .map_err(|e| ApiError::BadRequest(e))?;
    drop(manager);

    // Clone pool for background task
    let pool_clone = pool.inner().clone();
    let manager_clone = Arc::clone(job_manager.inner());

    // Spawn background task
    tokio::spawn(async move {
        let orchestrator = SyncOrchestrator::new(git_config, pool_clone, list_id);
        let result = orchestrator.run_sync(Arc::clone(&manager_clone)).await;

        // ... handle result (same as before) ...
    });

    Ok(Json(SyncStartResponse {
        job_id,
        message: format!("Sync job started for mailing list '{}'", slug),
    }))
}
```

2. **Add new `get_sync_status` endpoint** with slug parameter:
```rust
#[get("/admin/sync/<slug>/status")]
pub async fn get_sync_status(
    slug: String,
    job_manager: &State<Arc<Mutex<JobManager>>>,
) -> Result<Json<JobState>, ApiError> {
    // For now, we only have one global job manager
    // In production, you might want per-mailing-list job tracking
    let manager = job_manager.inner().lock().await;
    let state = manager.get_state().await;
    Ok(Json(state))
}
```

3. **Update route registration** in `src/main.rs`:
```rust
routes![
    // ... existing routes ...
    routes::admin::start_sync,
    routes::admin::get_sync_status,  // Update signature
    // ... other routes ...
]
```

---

### Phase 3: API Routes

#### Task 4: Create Mailing Lists CRUD Routes

**File**: `api-server/src/routes/mailing_lists.rs` (NEW FILE)

```rust
use rocket::serde::json::Json;
use rocket::{get, post, put, delete};
use rocket_db_pools::Connection;

use crate::db::LinuxKbDb;
use crate::error::ApiError;
use crate::models::{MailingList, MailingListRepository};
use crate::sync::{create_mailing_list_partitions, drop_mailing_list_partitions};

#[get("/mailing-lists")]
pub async fn list_mailing_lists(
    mut db: Connection<LinuxKbDb>,
) -> Result<Json<Vec<MailingList>>, ApiError> {
    let lists: Vec<MailingList> = sqlx::query_as(
        "SELECT * FROM mailing_lists ORDER BY sync_priority, name"
    )
    .fetch_all(&mut **db)
    .await?;

    Ok(Json(lists))
}

#[get("/mailing-lists/<slug>")]
pub async fn get_mailing_list(
    slug: String,
    mut db: Connection<LinuxKbDb>,
) -> Result<Json<MailingList>, ApiError> {
    let list: MailingList = sqlx::query_as(
        "SELECT * FROM mailing_lists WHERE slug = $1"
    )
    .bind(&slug)
    .fetch_one(&mut **db)
    .await
    .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;

    Ok(Json(list))
}

#[get("/mailing-lists/<slug>/repositories")]
pub async fn get_mailing_list_repos(
    slug: String,
    mut db: Connection<LinuxKbDb>,
) -> Result<Json<Vec<MailingListRepository>>, ApiError> {
    // Get mailing list ID
    let list_id: (i32,) = sqlx::query_as(
        "SELECT id FROM mailing_lists WHERE slug = $1"
    )
    .bind(&slug)
    .fetch_one(&mut **db)
    .await
    .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;

    // Get repositories
    let repos: Vec<MailingListRepository> = sqlx::query_as(
        "SELECT * FROM mailing_list_repositories
         WHERE mailing_list_id = $1 ORDER BY repo_order"
    )
    .bind(list_id.0)
    .fetch_all(&mut **db)
    .await?;

    Ok(Json(repos))
}

// Add POST, PUT, DELETE endpoints as needed for admin functionality
```

**Register routes** in `src/routes/mod.rs`:
```rust
pub mod mailing_lists;
```

**Register in** `src/main.rs`:
```rust
.mount(
    "/api",
    routes![
        // Mailing lists
        routes::mailing_lists::list_mailing_lists,
        routes::mailing_lists::get_mailing_list,
        routes::mailing_lists::get_mailing_list_repos,
        // ... existing routes ...
    ],
)
```

---

#### Task 5: Update Existing Routes with Mailing List Context

All existing routes need to accept a `slug` parameter and filter by `mailing_list_id`.

**Example**: `src/routes/threads.rs`

```rust
#[get("/<slug>/threads?<page>&<limit>&<sort_by>&<order>")]
pub async fn list_threads(
    slug: String,  // ADD THIS
    mut db: Connection<LinuxKbDb>,
    page: Option<i64>,
    limit: Option<i64>,
    sort_by: Option<String>,
    order: Option<String>,
) -> Result<Json<Vec<Thread>>, ApiError> {
    // Get mailing list ID
    let list_id: (i32,) = sqlx::query_as(
        "SELECT id FROM mailing_lists WHERE slug = $1"
    )
    .bind(&slug)
    .fetch_one(&mut **db)
    .await
    .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;

    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;

    // CHANGE: Add WHERE clause for mailing_list_id
    let threads: Vec<Thread> = sqlx::query_as(
        "SELECT * FROM threads
         WHERE mailing_list_id = $1
         ORDER BY last_date DESC
         LIMIT $2 OFFSET $3"
    )
    .bind(list_id.0)  // ADD THIS
    .bind(limit)
    .bind(offset)
    .fetch_all(&mut **db)
    .await?;

    Ok(Json(threads))
}
```

**Repeat this pattern** for all routes in:
- `src/routes/threads.rs`: `list_threads`, `search_threads`, `get_thread`
- `src/routes/emails.rs`: `get_email`
- `src/routes/authors.rs`: All endpoints
- `src/routes/stats.rs`: `get_stats`

**Route paths become**:
- `GET /api/:slug/threads`
- `GET /api/:slug/threads/:id`
- `GET /api/:slug/emails/:id`
- `GET /api/:slug/authors`
- `GET /api/:slug/stats`

---

### Phase 4: Frontend Updates

#### Task 6: Update API Client

**File**: `frontend/src/api/client.ts`

Add mailing list context to all API calls:

```typescript
// Add base method that includes mailing list slug
const apiCall = async <T>(
  mailingList: string,
  endpoint: string,
  options?: RequestInit
): Promise<T> => {
  const url = `${API_BASE_URL}/${mailingList}${endpoint}`;
  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });

  if (!response.ok) {
    throw new Error(`API error: ${response.statusText}`);
  }

  return response.json();
};

// Update all API methods
export const api = {
  threads: {
    list: (mailingList: string, params?: ThreadQueryParams) =>
      apiCall<Thread[]>(mailingList, `/threads?${new URLSearchParams(params)}`),

    get: (mailingList: string, id: number) =>
      apiCall<ThreadDetail>(mailingList, `/threads/${id}`),

    search: (mailingList: string, params: ThreadSearchParams) =>
      apiCall<Thread[]>(mailingList, `/threads/search?${new URLSearchParams(params)}`),
  },

  mailingLists: {
    list: () =>
      apiCall<MailingList[]>('', '/mailing-lists'),  // No slug prefix for this

    get: (slug: string) =>
      apiCall<MailingList>('', `/mailing-lists/${slug}`),
  },

  // ... update all other methods similarly ...
};
```

---

#### Task 7: Update Frontend Routing

**File**: `frontend/src/App.tsx`

```tsx
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        {/* Redirect root to default mailing list */}
        <Route path="/" element={<Navigate to="/bpf/threads" replace />} />

        {/* Mailing list routes */}
        <Route path="/:mailingList/threads" element={<MailLayout />} />
        <Route path="/:mailingList/threads/:id" element={<ThreadView />} />
        <Route path="/:mailingList/authors" element={<AuthorSearch />} />
        <Route path="/:mailingList/authors/:id" element={<AuthorProfile />} />

        {/* Settings (not scoped to mailing list) */}
        <Route path="/settings" element={<Settings />} />
      </Routes>
    </BrowserRouter>
  );
}
```

---

#### Task 8: Create Mailing List Selector Component

**File**: `frontend/src/components/MailingListSelector.tsx` (NEW)

```tsx
import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { api } from '../api/client';
import type { MailingList } from '../types';

export function MailingListSelector() {
  const { mailingList: currentSlug } = useParams<{ mailingList: string }>();
  const navigate = useNavigate();
  const [lists, setLists] = useState<MailingList[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadMailingLists();
  }, []);

  const loadMailingLists = async () => {
    try {
      const data = await api.mailingLists.list();
      setLists(data);
    } catch (error) {
      console.error('Failed to load mailing lists:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleChange = (slug: string) => {
    // Navigate to threads page of selected mailing list
    navigate(`/${slug}/threads`);
  };

  if (loading) {
    return <div className="text-sm text-gray-500">Loading...</div>;
  }

  return (
    <div className="relative">
      <label htmlFor="mailing-list" className="sr-only">
        Select Mailing List
      </label>
      <select
        id="mailing-list"
        value={currentSlug || ''}
        onChange={(e) => handleChange(e.target.value)}
        className="block w-full px-3 py-2 bg-white border border-gray-300
                   rounded-md shadow-sm focus:outline-none focus:ring-2
                   focus:ring-blue-500 focus:border-blue-500"
      >
        {lists.map((list) => (
          <option key={list.id} value={list.slug}>
            {list.name}
          </option>
        ))}
      </select>
    </div>
  );
}
```

---

#### Task 9: Update MailLayout to Include Selector

**File**: `frontend/src/components/MailLayout.tsx`

Add the selector to the navigation/header:

```tsx
import { MailingListSelector } from './MailingListSelector';

export function MailLayout() {
  return (
    <div className="min-h-screen bg-gray-50">
      <nav className="bg-white border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <div className="flex items-center space-x-4">
              <h1 className="text-xl font-bold">Linux KB</h1>

              {/* Mailing List Selector */}
              <div className="w-48">
                <MailingListSelector />
              </div>
            </div>

            {/* ... rest of nav ... */}
          </div>
        </div>
      </nav>

      {/* ... rest of layout ... */}
    </div>
  );
}
```

---

#### Task 10: Update Settings Page

**File**: `frontend/src/pages/Settings.tsx`

Add a new mailing list management panel:

```tsx
import { MailingListPanel } from '../components/settings/MailingListPanel';

export function Settings() {
  return (
    <div className="max-w-5xl mx-auto p-6">
      <h1 className="text-3xl font-bold mb-8">Settings</h1>

      <div className="space-y-6">
        <TimezonePanel />
        <MailingListPanel />  {/* NEW */}
        <SyncPanel />
        <DatabasePanel />
        <ConfigPanel />
      </div>
    </div>
  );
}
```

**File**: `frontend/src/components/settings/MailingListPanel.tsx` (NEW)

```tsx
import { useState, useEffect } from 'react';
import { api } from '../../api/client';
import type { MailingList } from '../../types';

export function MailingListPanel() {
  const [lists, setLists] = useState<MailingList[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadMailingLists();
  }, []);

  const loadMailingLists = async () => {
    try {
      const data = await api.mailingLists.list();
      setLists(data);
    } finally {
      setLoading(false);
    }
  };

  const startSync = async (slug: string) => {
    try {
      await api.admin.sync.start(slug);
      alert(`Sync started for ${slug}`);
    } catch (error) {
      alert(`Failed to start sync: ${error}`);
    }
  };

  return (
    <div className="bg-white rounded-lg border border-gray-200 p-6">
      <h2 className="text-xl font-semibold mb-6">Mailing Lists</h2>

      {loading ? (
        <div className="text-center py-8">Loading...</div>
      ) : (
        <div className="space-y-4">
          {lists.map((list) => (
            <div
              key={list.id}
              className="flex items-center justify-between p-4
                         border border-gray-200 rounded-lg"
            >
              <div>
                <h3 className="font-medium">{list.name}</h3>
                <p className="text-sm text-gray-500">{list.description}</p>
                {list.last_synced_at && (
                  <p className="text-xs text-gray-400 mt-1">
                    Last synced: {new Date(list.last_synced_at).toLocaleString()}
                  </p>
                )}
              </div>

              <div className="flex items-center space-x-3">
                <span
                  className={`px-2 py-1 text-xs rounded ${
                    list.enabled
                      ? 'bg-green-100 text-green-800'
                      : 'bg-gray-100 text-gray-800'
                  }`}
                >
                  {list.enabled ? 'Enabled' : 'Disabled'}
                </span>

                <button
                  onClick={() => startSync(list.slug)}
                  disabled={!list.enabled}
                  className="px-3 py-1 bg-blue-600 text-white text-sm
                           rounded hover:bg-blue-700 disabled:opacity-50
                           disabled:cursor-not-allowed"
                >
                  Sync
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
```

---

#### Task 11: Update TypeScript Types

**File**: `frontend/src/types/index.ts`

Add mailing list types:

```typescript
export interface MailingList {
  id: number;
  name: string;
  slug: string;
  description: string | null;
  enabled: boolean;
  sync_priority: number;
  created_at: string;
  last_synced_at: string | null;
}

export interface MailingListRepository {
  id: number;
  mailing_list_id: number;
  repo_url: string;
  repo_order: number;
  created_at: string;
}

// Update existing types to include mailing_list_id
export interface Thread {
  id: number;
  mailing_list_id: number;  // ADD
  root_message_id: string;
  subject: string;
  start_date: string;
  last_date: string;
  message_count: number;
}

export interface Email {
  id: number;
  mailing_list_id: number;  // ADD
  message_id: string;
  // ... rest of fields
}

// ... update all other types similarly ...
```

---

### Phase 5: Testing & Cleanup

#### Task 12: Test Database Reset

```bash
cd api-server
cargo run --release
```

Then call the reset endpoint:
```bash
curl -X POST http://localhost:8000/api/admin/database/reset
```

Verify:
- Tables created with partitions
- `mailing_lists` has BPF and sched-ext
- `mailing_list_repositories` has default repos
- Partitions exist: `authors_bpf`, `emails_bpf`, etc.

---

#### Task 13: Test Sync Flow

1. Start sync for BPF:
```bash
curl -X POST http://localhost:8000/api/admin/sync/bpf/start
```

2. Check status:
```bash
curl http://localhost:8000/api/admin/sync/bpf/status
```

3. Verify data imported into partitioned tables:
```sql
SELECT COUNT(*) FROM emails_bpf;
SELECT COUNT(*) FROM threads_bpf;
```

---

#### Task 14: Clean Up Old Documentation

**Delete these files**:
- `MAILING_LIST_BROWSER.md`
- `SYNC_PIPELINE_REFACTOR.md`
- `THREADING_IMPROVEMENTS.md`

---

## Compilation & Testing Checklist

After completing all tasks:

1. **Backend compilation**:
```bash
cd api-server
cargo check
cargo test
```

2. **Frontend compilation**:
```bash
cd frontend
npm run build
npm run lint
```

3. **Integration test**:
   - Reset database
   - Sync BPF mailing list
   - Verify frontend loads with mailing list selector
   - Switch between mailing lists
   - Search/browse threads

---

## Key Architecture Points

### Database Partitioning
- PostgreSQL LIST partitioning by `mailing_list_id`
- Each mailing list gets its own partitions
- Queries automatically route to correct partition
- Enables constant performance at scale

### Multi-Repository Support
- `mailing_list_repositories` table with `repo_order`
- GitManager syncs all repos sequentially
- Commits aggregated from all repos
- Each commit tracked with `(hash, path, repo_order)`

### Mailing List Context
- All routes prefixed with `/:slug/`
- All queries filtered by `mailing_list_id`
- Frontend maintains current mailing list in URL
- API client includes mailing list in all calls

---

## Common Pitfalls

1. **Forgetting `mailing_list_id`**: Every INSERT and SELECT must include it
2. **Unique constraints**: Now need `(mailing_list_id, field)` pairs
3. **Foreign keys**: Check partitioning compatibility
4. **Index names**: Include slug to avoid conflicts (e.g., `idx_emails_bpf_date`)
5. **SQL injection**: Use parameterized queries, especially with `slug`

---

## Environment Variables

Update your `.env` or set these:

```bash
# Database
DATABASE_URL=postgres://postgres:example@localhost:5432/linux-kernel-kb

# Git mirror storage
MIRROR_BASE_PATH=./mirrors

# Logging
RUST_LOG=info
```

---

## Production Considerations

For production deployment:

1. **Job Management**: Implement per-mailing-list job tracking
2. **Rate Limiting**: Add rate limits to sync endpoints
3. **Webhooks**: Add webhook support for sync completion
4. **Metrics**: Add Prometheus metrics for sync performance
5. **Error Handling**: Improve error messages and recovery
6. **Authentication**: Add auth to admin endpoints
7. **Caching**: Consider Redis for frequently accessed data

---

## Success Criteria

You'll know you're done when:

- ✅ `cargo check` passes
- ✅ Database reset creates partitioned schema
- ✅ Can sync BPF and sched-ext independently
- ✅ Frontend displays mailing list selector
- ✅ Can switch between mailing lists
- ✅ Threads/emails load correctly per mailing list
- ✅ All data isolated by partition
- ✅ Can add new mailing lists via API

---

## Getting Help

If you get stuck:

1. Check the database schema in `src/sync/mod.rs::reset_database()`
2. Review the Git module tests in `src/sync/git.rs`
3. Look at existing route patterns for query structure
4. Check SQLx documentation for partition compatibility
5. Review PostgreSQL partitioning docs for syntax

Good luck! The foundation is solid - you're just wiring it all together now. 🚀

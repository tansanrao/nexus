# Nexus Search with Meilisearch — **Final Implementation Plan**

> **Scope update (per requirements):**
>
> * We will **only** index and search **threads** (no individual email/doc index).
> * Keep a separate **authors** index for people lookup and faceting.
> * **Embeddings:** use **Qwen3-Embedding-0.6B** generated **locally** via Hugging Face Inference.
> * **Search mode:** always **hybrid** (lexical + semantic) with an **advanced slider** to adjust `semanticRatio`.
> * **Backend only:** Meilisearch is **not exposed** publicly; UI hits the **api-server**, which talks to Meili over the private network.
> * **Migration:** we are in dev; **remove** the existing Postgres search implementation and **wire Meili in one go**.
> * **Results:** search results are **always distinct by `thread_id`** (one document per thread).

---

## 1) Architecture

```
Postgres (source of truth) ──► Indexer (Rust)
        ▲                             │
        │                             ├── Meilisearch CE (private network)
        │                             │     - indexes: threads, authors
  Parser + JWZ                        │     - hybrid search (lexical+semantic)
        └─ Thread builder             │
                                      └── Local HF Inference (Qwen3-Embedding-0.6B)
```

* **Parser & JWZ** stay as-is. During indexing we derive per-thread **`discussion_text`** (quotes/patches removed) from member emails.
* **Meili** stores thread & author docs. **Hybrid** queries are handled directly by Meili; no extra app-side rerank.
* **API server** owns all calls to Meili and enforces auth, filters, and query shaping.

---

## 2) Index design

### 2.1 `threads` (primary search index)

**Primary key:** `thread_id` (one doc per thread)

**Document shape**

```jsonc
{
  "thread_id": 7890,
  "mailing_list": "linux-kernel",
  "root_message_id": "<...>",
  "subject": "mm: fix xyz",
  "normalized_subject": "mm: fix xyz",
  "start_date": "2025-09-25T12:34:56Z",
  "last_date": "2025-09-26T08:11:00Z",
  "message_count": 18,
  "participants": ["Alice Foo", "Bob Bar"],
  "participant_ids": [2, 5],
  "has_patches": true,
  "series_id": "abcd1234",
  "series_number": 3,
  "series_total": 7,
  "discussion_text": "Concatenation of non-quoted, non-patch discussion from key messages...",
  "_vector": [ /* optional: only if storing with docs */ ]
}
```

**Settings**

* `searchableAttributes`: `["subject", "discussion_text", "participants"]`
* `filterableAttributes`: `["mailing_list","participant_ids","has_patches","series_id","start_date","last_date","message_count"]`
* `sortableAttributes`: `["last_date","start_date","message_count"]`
* `distinctAttribute`: `"thread_id"` (safety; each thread is unique)

> **Note:** With one-doc-per-thread, distinct is naturally satisfied; setting it removes any risk if we ever introduce variants.

### 2.2 `authors` (people lookup + facets)

**Primary key:** `author_id`

**Document shape**

```json
{
  "author_id": 2,
  "canonical_name": "Alice Foo",
  "email": "alice@example.com",
  "aliases": ["A. Foo"],
  "mailing_lists": ["linux-kernel", "netdev"],
  "first_seen": "2023-06-11T00:00:00Z",
  "last_seen": "2025-10-01T00:00:00Z",
  "thread_count": 421,
  "email_count": 1180
}
```

**Settings**

* `searchableAttributes`: `["canonical_name", "aliases", "email"]`
* `filterableAttributes`: `["mailing_lists"]`
* `sortableAttributes`: `["last_seen","thread_count","email_count"]`

---

## 3) Producing `discussion_text` (thread-level)

We no longer index individual emails. Instead, we compute a **single `discussion_text`** per thread:

1. For each email in the thread:

   * Use existing patch analysis to **remove**:

     * Inline diff sections, diffstat, trailers (Signed-off-by, Reviewed-by, etc.), footer lines after `--`.
     * **Quoted text** (`>` lines) and reply headers (`On ... wrote:`) with their quoted blocks.
   * The helper `make_discussion_text(mail)` (from previous plan) returns a clean body.
2. **Aggregation strategy** (to keep docs compact & relevant):

   * Take the **root** message’s clean text + the first **K** replies that add substantive (non-empty) discussion (e.g., `K=5`), concatenated with separators.
   * Hard cap to **N characters** (e.g., 16–24k) to bound embedding & indexing costs; prefer earlier, higher-signal messages.
   * Store a tiny **`first_post_excerpt`** separately for result snippets if desired.

This yields a crisp, patch/quote-free summary of thread discourse for both lexical and semantic matching.

---

## 4) Embeddings (Qwen3-Embedding-0.6B, local)

* Run **Qwen3-Embedding-0.6B** behind a local HF Inference server/sidecar.
* Use a **single embedding** per thread over `normalized_subject + "\n\n" + discussion_text`.
* **Dimension:** Qwen3 supports **configurable output sizes** (commonly up to **1024**). Pick a single global dimension (e.g., **1024**) and keep it consistent with Meili’s embedder setting.
* **When indexing**:

  * Either upload vectors inline with docs (Meili `userProvided` embedder), or
  * Configure Meili to call your internal **REST embedder**; with local inference this is a simple HTTP hop.

---

## 5) Meilisearch configuration (self‑hosted CE)

**Indexes**

* Create `threads` (primary), `authors`.

**Embedders**

* Configure **`userProvided`** (or **`rest`**) embedder for `threads` with the chosen dimension (e.g., `1024`).

**Settings per index**

* Apply the `searchable`, `filterable`, `sortable` attributes listed above.
* For `threads`, set `distinctAttribute` to `thread_id`.

**Backups**

* Enable **snapshots/dumps** on a schedule; store alongside Postgres backups.

**Network**

* Bind Meili to **private** interface; only the **api-server** can reach it.

---

## 6) API surface (backend-owned; UI never talks to Meili directly)

### 6.1 Thread search (always **hybrid**)

`GET /api/v1/:slug/threads/search`

**Query params**

* `q`: text query
* `semanticRatio`: float (0..1). **Always present; default 0.35**. Expose as an **advanced slider** in the UI.
* `filters`: DSL mapped to Meili `filter` (e.g., list, participants, has_patches, date range)
* `sort`: `["last_date:desc"]` by default; allow alternatives in UI (advanced)
* `limit`, `offset`

**Meili request (conceptual)**

```jsonc
{
  "q": "virtio net rx slow",
  "vector": [ /* query vector from Qwen3 (local) */ ],
  "hybrid": { "embedder": "threads-qwen3", "semanticRatio": 0.35 },
  "filter": "mailing_list = 'linux-kernel'",
  "sort": ["last_date:desc"],
  "limit": 20,
  "attributesToHighlight": ["subject","discussion_text"],
  "attributesToCrop": ["discussion_text"]
}
```

**Always distinct by thread**: with one-doc-per-thread and `distinctAttribute: thread_id`, the result set is inherently unique per thread.

### 6.2 Author search

`GET /api/v1/authors/search` → direct Meili search on `authors` (lexical only is fine, or add a small embedder later if desired). Filters: by `mailing_lists`.

---

## 7) Indexing pipeline (Rust)

1. **Thread rollup**: after JWZ threading, build `discussion_text` using the stripping rules and aggregation.
2. **Embedding**: generate Qwen3 vector for `subject + discussion_text`.
3. **Upsert** doc into `threads` (and update `authors` counts if needed).
4. **Batches**: upload in chunks (1–5k) and track Meili tasks until processed.
5. **Updates**: on new replies, recompute thread `discussion_text` incrementally, re-embed, and update the doc; bump `last_date`, `message_count`, `participants`.

---

## 8) Frontend behavior

* **Hybrid by default**; expose **Semantic strength** slider (maps to `semanticRatio`).
* Show facets (mailing list, participants, has patches, date range).
* Present highlights (`_formatted`) for `subject` / cropped `discussion_text`.
* Clicking a result opens the **full thread** view (server fetch, not Meili).

---

## 9) Security, tenancy, and ops

* Meili runs on a **private network**; only the API may access it. Consider **tenant tokens** internally if you later want per-user scopes while still proxying via the API.
* Schedule **snapshots/dumps** for index backup.
* Instrument indexing latency, Meili task failures, and search P95.

---

## 10) Migration (dev only)

* **Remove** Postgres FTS/pgvector search code and associated migrations.
* **Introduce** Meili client and settings.
* Build the two indexes from scratch; backfill from Postgres.
* Switch the UI search to call the new endpoint.

---

## 11) Acceptance & quality checks

* Golden queries (100–200) covering conceptual and exact lookups.
* **No patch/quote leakage:** assert results never contain `diff --git`, `+++`, `@@`, or trailer lines like `Signed-off-by:` in snippets.
* Latency target: sub‑100 ms p95 for typical queries; measure with warm cache.

---

## 12) Copy‑paste snippets

### 12.1 Create `threads` index & settings

```bash
curl -X POST "$MEILI/indexes" -H 'Content-Type: application/json' \
  --data '{"uid":"threads","primaryKey":"thread_id"}'

curl -X PUT "$MEILI/indexes/threads/settings/searchable-attributes" \
  -H 'Content-Type: application/json' \
  --data '["subject","discussion_text","participants"]'

curl -X PUT "$MEILI/indexes/threads/settings/filterable-attributes" \
  -H 'Content-Type: application/json' \
  --data '["mailing_list","participant_ids","has_patches","series_id","start_date","last_date","message_count"]'

curl -X PUT "$MEILI/indexes/threads/settings/sortable-attributes" \
  -H 'Content-Type: application/json' \
  --data '["last_date","start_date","message_count"]'

curl -X PUT "$MEILI/indexes/threads/settings/distinct-attribute" \
  -H 'Content-Type: application/json' \
  --data '"thread_id"'
```

### 12.2 Configure `userProvided` embedder for threads (dimension = 1024 example)

```bash
curl -X PATCH "$MEILI/indexes/threads/settings/embedders" \
  -H 'Content-Type: application/json' \
  --data '{
    "threads-qwen3": { "source": "userProvided", "dimensions": 1024 }
  }'
```

### 12.3 Hybrid search request (from the API server)

```jsonc
POST /indexes/threads/search
{
  "q": "page allocator regression",
  "vector": [ /* 1024 floats */ ],
  "hybrid": { "embedder": "threads-qwen3", "semanticRatio": 0.35 },
  "filter": "mailing_list = 'linux-kernel'",
  "sort": ["last_date:desc"],
  "limit": 20,
  "attributesToHighlight": ["subject","discussion_text"],
  "attributesToCrop": ["discussion_text"]
}
```

### 12.4 `authors` index (minimal)

```bash
curl -X POST "$MEILI/indexes" -H 'Content-Type: application/json' \
  --data '{"uid":"authors","primaryKey":"author_id"}'

curl -X PUT "$MEILI/indexes/authors/settings/searchable-attributes" \
  -H 'Content-Type: application/json' \
  --data '["canonical_name","aliases","email"]'

curl -X PUT "$MEILI/indexes/authors/settings/filterable-attributes" \
  -H 'Content-Type: application/json' \
  --data '["mailing_lists"]'

curl -X PUT "$MEILI/indexes/authors/settings/sortable-attributes" \
  -H 'Content-Type: application/json' \
  --data '["last_seen","thread_count","email_count"]'
```

---

## 13) What changes in the repo

* Remove old search adapters (Postgres FTS/pgvector) and routes.
* Add a `meili` client module, index bootstrapping task, and thread rollup + embed job.
* Replace `/threads/search` to call Meili **hybrid** with mandatory `semanticRatio` param (defaulted).
* Keep `authors` endpoints for people lookup and filters.

---

## 14) Open items / toggles

* **K** (reply count to aggregate): start with 5, tune by recall/latency.
* **N** (char cap of `discussion_text`): start at 24k; raise/lower based on embedding latency.
* **Default `semanticRatio`**: 0.35 (tune with golden queries).

-- Expand embedding dimension to 768 and add thread-level embeddings.

-- Drop existing vector index before changing the column.
DROP INDEX IF EXISTS idx_emails_embedding_hnsw;

-- Recreate the column with the new dimension. Existing data is dropped; run
-- re-embedding jobs after applying this migration.
ALTER TABLE emails DROP COLUMN IF EXISTS embedding;
ALTER TABLE emails ADD COLUMN embedding VECTOR(768);

-- Recreate the email embedding index with the updated dimension.
CREATE INDEX idx_emails_embedding_hnsw ON emails USING vchordrq (embedding vector_cosine_ops);

-- Thread-level embedding materialization for faster semantic search.
CREATE TABLE thread_embeddings (
    id SERIAL,
    mailing_list_id INTEGER NOT NULL,
    thread_id INTEGER NOT NULL,
    embedding VECTOR(768),
    email_count INTEGER NOT NULL DEFAULT 0,
    aggregated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, mailing_list_id),
    UNIQUE (mailing_list_id, thread_id),
    FOREIGN KEY (mailing_list_id, thread_id) REFERENCES threads(mailing_list_id, id) ON DELETE CASCADE
) PARTITION BY LIST (mailing_list_id);

CREATE TABLE thread_embeddings_default PARTITION OF thread_embeddings DEFAULT;

CREATE INDEX idx_thread_embeddings_thread_id ON thread_embeddings(thread_id);
CREATE INDEX idx_thread_embeddings_hnsw ON thread_embeddings USING vchordrq (embedding vector_cosine_ops);

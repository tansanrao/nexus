-- Revert thread-level embeddings and restore the original email embedding dimension.

DROP INDEX IF EXISTS idx_thread_embeddings_hnsw;
DROP INDEX IF EXISTS idx_thread_embeddings_thread_id;

DROP TABLE IF EXISTS thread_embeddings_default;
DROP TABLE IF EXISTS thread_embeddings;

DROP INDEX IF EXISTS idx_emails_embedding_hnsw;

ALTER TABLE emails DROP COLUMN IF EXISTS embedding;
ALTER TABLE emails ADD COLUMN embedding VECTOR(384);

CREATE INDEX idx_emails_embedding_hnsw ON emails USING vchordrq (embedding vector_cosine_ops);

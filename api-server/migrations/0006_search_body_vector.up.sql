ALTER TABLE emails
    ADD COLUMN IF NOT EXISTS search_body TEXT;

UPDATE emails
SET search_body = body
WHERE search_body IS NULL;

ALTER TABLE threads
    ADD COLUMN IF NOT EXISTS search_vector TSVECTOR;

CREATE INDEX IF NOT EXISTS idx_threads_search_vector
    ON threads USING GIN (search_vector);

CREATE INDEX IF NOT EXISTS idx_threads_mailing_last_date
    ON threads (mailing_list_id, last_date DESC);

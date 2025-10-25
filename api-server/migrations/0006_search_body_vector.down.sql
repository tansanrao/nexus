DROP INDEX IF EXISTS idx_threads_mailing_last_date;
DROP INDEX IF EXISTS idx_threads_search_vector;
ALTER TABLE threads
    DROP COLUMN IF EXISTS search_vector;
ALTER TABLE emails
    DROP COLUMN IF EXISTS search_body;

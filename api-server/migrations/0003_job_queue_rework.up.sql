-- Rework job queue schema to support typed jobs and normalized statuses.

CREATE TYPE job_type AS ENUM ('import', 'embedding_refresh', 'index_maintenance');
CREATE TYPE job_status AS ENUM ('queued', 'running', 'succeeded', 'failed', 'cancelled');

ALTER TABLE sync_jobs RENAME TO jobs;

ALTER TABLE jobs
    ADD COLUMN job_type job_type NOT NULL DEFAULT 'import',
    ADD COLUMN status job_status NOT NULL DEFAULT 'queued',
    ADD COLUMN payload JSONB NOT NULL DEFAULT '{}',
    ADD COLUMN last_heartbeat TIMESTAMPTZ;

-- Migrate legacy phase values to the new status column.
UPDATE jobs SET status = 'queued' WHERE status = 'queued' AND phase = 'waiting';
UPDATE jobs SET status = 'running' WHERE phase IN ('parsing', 'threading');
UPDATE jobs SET status = 'succeeded' WHERE phase = 'done';
UPDATE jobs SET status = 'failed' WHERE phase = 'errored';

-- Remove legacy phase column now that state migration is complete.
ALTER TABLE jobs DROP COLUMN phase;

-- Rename existing indexes to reflect the new table name and create additional helpers.
DROP INDEX IF EXISTS idx_sync_jobs_phase_priority;
DROP INDEX IF EXISTS idx_sync_jobs_mailing_list_id;

CREATE INDEX idx_jobs_status_priority ON jobs(status, priority DESC, created_at);
CREATE INDEX idx_jobs_mailing_list_id ON jobs(mailing_list_id);
CREATE INDEX idx_jobs_job_type_status ON jobs(job_type, status);

-- Ensure default expressions no longer rely on the legacy column (defaults already inline).

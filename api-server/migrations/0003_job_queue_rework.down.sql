-- Revert job queue schema changes back to the legacy sync_jobs structure.

ALTER TABLE jobs ADD COLUMN phase TEXT;

-- Restore legacy phase values based on the normalized status column.
UPDATE jobs SET phase = 'waiting' WHERE status = 'queued' AND phase IS NULL;
UPDATE jobs SET phase = 'parsing' WHERE status = 'running' AND phase IS NULL;
UPDATE jobs SET phase = 'done' WHERE status = 'succeeded' AND phase IS NULL;
UPDATE jobs SET phase = 'errored' WHERE status = 'failed' AND phase IS NULL;
UPDATE jobs SET phase = 'errored' WHERE status = 'cancelled' AND phase IS NULL;

-- Remove new columns before renaming table.
ALTER TABLE jobs
    DROP COLUMN job_type,
    DROP COLUMN status,
    DROP COLUMN payload,
    DROP COLUMN last_heartbeat;

-- Reinstate legacy constraint on phase values.
ALTER TABLE jobs
    ALTER COLUMN phase SET NOT NULL,
    ALTER COLUMN phase SET DEFAULT 'waiting',
    ADD CONSTRAINT sync_jobs_phase_check CHECK (phase IN ('waiting', 'parsing', 'threading', 'done', 'errored'));

-- Restore defaults for phase and priority ordering indexes.
DROP INDEX IF EXISTS idx_jobs_status_priority;
DROP INDEX IF EXISTS idx_jobs_mailing_list_id;
DROP INDEX IF EXISTS idx_jobs_job_type_status;

CREATE INDEX idx_sync_jobs_phase_priority ON jobs(phase, priority DESC, created_at);
CREATE INDEX idx_sync_jobs_mailing_list_id ON jobs(mailing_list_id);

ALTER TABLE jobs RENAME TO sync_jobs;

DROP TYPE IF EXISTS job_status;
DROP TYPE IF EXISTS job_type;

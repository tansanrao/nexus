-- Remove embedding-related job type and cleanup existing records
DELETE FROM jobs WHERE job_type = 'embedding_refresh';

ALTER TABLE jobs ALTER COLUMN job_type DROP DEFAULT;
ALTER TYPE job_type RENAME TO job_type_old;
CREATE TYPE job_type AS ENUM ('import', 'index_maintenance');
ALTER TABLE jobs ALTER COLUMN job_type TYPE job_type USING job_type::text::job_type;
ALTER TABLE jobs ALTER COLUMN job_type SET DEFAULT 'import';
DROP TYPE job_type_old;

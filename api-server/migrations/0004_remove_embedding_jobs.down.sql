-- Restore embedding_refresh job type variant
ALTER TABLE jobs ALTER COLUMN job_type DROP DEFAULT;
ALTER TYPE job_type RENAME TO job_type_old;
CREATE TYPE job_type AS ENUM ('import', 'embedding_refresh', 'index_maintenance');
ALTER TABLE jobs ALTER COLUMN job_type TYPE job_type USING job_type::text::job_type;
ALTER TABLE jobs ALTER COLUMN job_type SET DEFAULT 'import';
DROP TYPE job_type_old;

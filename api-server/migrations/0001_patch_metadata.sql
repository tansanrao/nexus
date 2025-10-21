-- Add columns to capture git patch detection metadata.

CREATE TYPE patch_type AS ENUM ('none', 'inline', 'attachment');

ALTER TABLE emails
    ADD COLUMN patch_type patch_type NOT NULL DEFAULT 'none',
    ADD COLUMN is_patch_only BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN patch_metadata JSONB;

CREATE INDEX idx_emails_patch_type ON emails(patch_type);
CREATE INDEX idx_emails_is_patch_only ON emails(is_patch_only);

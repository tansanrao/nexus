-- Allow index maintenance jobs without a mailing list id
ALTER TABLE jobs ALTER COLUMN mailing_list_id DROP NOT NULL;

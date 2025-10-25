-- Restore non-null constraint on mailing_list_id
DELETE FROM jobs WHERE mailing_list_id IS NULL;
ALTER TABLE jobs ALTER COLUMN mailing_list_id SET NOT NULL;

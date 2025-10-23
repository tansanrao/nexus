-- Rollback for fresh initial schema
-- Drops tables, partitions, types, and extensions created by 0001_initial.up.sql.

-- Auth & notifications
DROP TABLE IF EXISTS notification_cursors;
DROP INDEX IF EXISTS idx_notifications_user_created;
DROP TABLE IF EXISTS notifications;
DROP TABLE IF EXISTS user_thread_follows;
DROP INDEX IF EXISTS user_refresh_tokens_user_idx;
DROP TABLE IF EXISTS user_refresh_tokens;
DROP TABLE IF EXISTS user_profiles;
DROP TABLE IF EXISTS local_user_credentials;
DROP INDEX IF EXISTS idx_users_oidc;
DROP TABLE IF EXISTS users;

-- Partitioned tables (drop partitions first to avoid dependency issues)
DROP TABLE IF EXISTS thread_memberships_default;
DROP INDEX IF EXISTS idx_thread_memberships_email_id;
DROP INDEX IF EXISTS idx_thread_memberships_thread_id;
DROP TABLE IF EXISTS thread_memberships;
DROP TABLE IF EXISTS email_references_default;
DROP INDEX IF EXISTS idx_email_references_ref_msg_id;
DROP INDEX IF EXISTS idx_email_references_email_id;
DROP TABLE IF EXISTS email_references;
DROP TABLE IF EXISTS email_recipients_default;
DROP INDEX IF EXISTS idx_email_recipients_author_id;
DROP INDEX IF EXISTS idx_email_recipients_email_id;
DROP TABLE IF EXISTS email_recipients;
DROP TABLE IF EXISTS threads_default;
DROP INDEX IF EXISTS idx_threads_message_count;
DROP INDEX IF EXISTS idx_threads_last_date;
DROP INDEX IF EXISTS idx_threads_start_date;
DROP TABLE IF EXISTS threads;
DROP TABLE IF EXISTS emails_default;
DROP INDEX IF EXISTS idx_emails_embedding_hnsw;
DROP INDEX IF EXISTS idx_emails_subject_trgm;
DROP INDEX IF EXISTS idx_emails_body_ts;
DROP INDEX IF EXISTS idx_emails_lex_ts;
DROP INDEX IF EXISTS idx_emails_unthreaded;
DROP INDEX IF EXISTS idx_emails_threaded_at;
DROP INDEX IF EXISTS idx_emails_series_id;
DROP INDEX IF EXISTS idx_emails_normalized_subject;
DROP INDEX IF EXISTS idx_emails_in_reply_to;
DROP INDEX IF EXISTS idx_emails_date;
DROP INDEX IF EXISTS idx_emails_author_id;
DROP TABLE IF EXISTS emails;

-- Metadata & configuration tables
DROP INDEX IF EXISTS idx_author_activity_last_date;
DROP INDEX IF EXISTS idx_author_activity_mailing_list;
DROP TABLE IF EXISTS author_mailing_list_activity;
DROP INDEX IF EXISTS idx_author_name_aliases_author_id;
DROP TABLE IF EXISTS author_name_aliases;
DROP TABLE IF EXISTS authors;
DROP INDEX IF EXISTS idx_mailing_list_repos_list_id;
DROP TABLE IF EXISTS mailing_list_repositories;
DROP INDEX IF EXISTS idx_mailing_lists_enabled;
DROP INDEX IF EXISTS idx_mailing_lists_slug;
DROP TABLE IF EXISTS mailing_lists;
DROP INDEX IF EXISTS idx_sync_jobs_mailing_list_id;
DROP INDEX IF EXISTS idx_sync_jobs_phase_priority;
DROP TABLE IF EXISTS sync_jobs;

-- Types
DROP TYPE IF EXISTS patch_type;

-- Extensions
DROP EXTENSION IF EXISTS vchord CASCADE;
DROP EXTENSION IF EXISTS pg_trgm;

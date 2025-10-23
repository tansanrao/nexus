-- Fresh initial schema for Nexus (DB refactor)
-- Installs required extensions, enums, tables, and indexes for search/auth/notifications.

-- ============================================================================
-- Extensions
-- ============================================================================
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS vchord;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- ============================================================================
-- Types
-- ============================================================================
CREATE TYPE patch_type AS ENUM ('none', 'inline', 'attachment');

-- ============================================================================
-- Metadata & configuration tables
-- ============================================================================
CREATE TABLE sync_jobs (
    id SERIAL PRIMARY KEY,
    mailing_list_id INTEGER NOT NULL,
    phase TEXT NOT NULL DEFAULT 'waiting' CHECK (phase IN ('waiting', 'parsing', 'threading', 'done', 'errored')),
    priority INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT
);

CREATE INDEX idx_sync_jobs_phase_priority ON sync_jobs(phase, priority DESC, created_at);
CREATE INDEX idx_sync_jobs_mailing_list_id ON sync_jobs(mailing_list_id);

CREATE TABLE mailing_lists (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    enabled BOOLEAN DEFAULT true,
    sync_priority INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_synced_at TIMESTAMPTZ,
    last_threaded_at TIMESTAMPTZ
);

CREATE INDEX idx_mailing_lists_slug ON mailing_lists(slug);
CREATE INDEX idx_mailing_lists_enabled ON mailing_lists(enabled);

CREATE TABLE mailing_list_repositories (
    id SERIAL PRIMARY KEY,
    mailing_list_id INTEGER REFERENCES mailing_lists(id) ON DELETE CASCADE,
    repo_url TEXT NOT NULL,
    repo_order INTEGER NOT NULL DEFAULT 0,
    last_indexed_commit TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (mailing_list_id, repo_order)
);

CREATE INDEX idx_mailing_list_repos_list_id ON mailing_list_repositories(mailing_list_id);

CREATE TABLE authors (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    canonical_name TEXT,
    first_seen TIMESTAMPTZ DEFAULT NOW(),
    last_seen TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE author_name_aliases (
    id SERIAL PRIMARY KEY,
    author_id INTEGER NOT NULL REFERENCES authors(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    usage_count INTEGER DEFAULT 1,
    first_seen TIMESTAMPTZ DEFAULT NOW(),
    last_seen TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (author_id, name)
);

CREATE INDEX idx_author_name_aliases_author_id ON author_name_aliases(author_id);

CREATE TABLE author_mailing_list_activity (
    author_id INTEGER NOT NULL REFERENCES authors(id) ON DELETE CASCADE,
    mailing_list_id INTEGER NOT NULL REFERENCES mailing_lists(id) ON DELETE CASCADE,
    first_email_date TIMESTAMPTZ,
    last_email_date TIMESTAMPTZ,
    email_count BIGINT DEFAULT 0,
    thread_count BIGINT DEFAULT 0,
    PRIMARY KEY (author_id, mailing_list_id)
);

CREATE INDEX idx_author_activity_mailing_list ON author_mailing_list_activity(mailing_list_id);
CREATE INDEX idx_author_activity_last_date ON author_mailing_list_activity(last_email_date);

-- ============================================================================
-- Partitioned tables (per mailing_list_id)
-- ============================================================================
CREATE TABLE emails (
    id SERIAL,
    mailing_list_id INTEGER NOT NULL,
    message_id TEXT NOT NULL,
    git_commit_hash TEXT NOT NULL,
    author_id INTEGER NOT NULL,
    subject TEXT NOT NULL,
    normalized_subject TEXT,
    date TIMESTAMPTZ NOT NULL,
    in_reply_to TEXT,
    body TEXT,
    series_id TEXT,
    series_number INTEGER,
    series_total INTEGER,
    epoch INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    threaded_at TIMESTAMPTZ,
    patch_type patch_type NOT NULL DEFAULT 'none',
    is_patch_only BOOLEAN NOT NULL DEFAULT false,
    patch_metadata JSONB,
    embedding VECTOR(384),
    lex_ts TSVECTOR,
    body_ts TSVECTOR,
    PRIMARY KEY (id, mailing_list_id),
    UNIQUE (mailing_list_id, message_id),
    UNIQUE (mailing_list_id, git_commit_hash)
) PARTITION BY LIST (mailing_list_id);

CREATE INDEX idx_emails_author_id ON emails(author_id);
CREATE INDEX idx_emails_date ON emails(date);
CREATE INDEX idx_emails_in_reply_to ON emails(in_reply_to);
CREATE INDEX idx_emails_normalized_subject ON emails(normalized_subject);
CREATE INDEX idx_emails_series_id ON emails(series_id);
CREATE INDEX idx_emails_threaded_at ON emails(threaded_at);
CREATE INDEX idx_emails_unthreaded ON emails(id) WHERE threaded_at IS NULL;
CREATE INDEX idx_emails_lex_ts ON emails USING GIN (lex_ts);
CREATE INDEX idx_emails_body_ts ON emails USING GIN (body_ts);
CREATE INDEX idx_emails_subject_trgm ON emails USING GIN (subject gin_trgm_ops);
CREATE INDEX idx_emails_embedding_hnsw ON emails USING vchordrq (embedding vector_cosine_ops);

CREATE TABLE emails_default PARTITION OF emails DEFAULT;

CREATE TABLE threads (
    id SERIAL,
    mailing_list_id INTEGER NOT NULL,
    root_message_id TEXT NOT NULL,
    subject TEXT NOT NULL,
    start_date TIMESTAMPTZ NOT NULL,
    last_date TIMESTAMPTZ NOT NULL,
    message_count INTEGER DEFAULT 0,
    membership_hash BYTEA,
    PRIMARY KEY (id, mailing_list_id),
    UNIQUE (mailing_list_id, root_message_id)
) PARTITION BY LIST (mailing_list_id);

CREATE INDEX idx_threads_start_date ON threads(start_date);
CREATE INDEX idx_threads_last_date ON threads(last_date);
CREATE INDEX idx_threads_message_count ON threads(message_count);

CREATE TABLE threads_default PARTITION OF threads DEFAULT;

CREATE TABLE email_recipients (
    id SERIAL,
    mailing_list_id INTEGER NOT NULL,
    email_id INTEGER NOT NULL,
    author_id INTEGER NOT NULL,
    recipient_type TEXT CHECK (recipient_type IN ('to', 'cc')),
    PRIMARY KEY (id, mailing_list_id)
) PARTITION BY LIST (mailing_list_id);

CREATE INDEX idx_email_recipients_email_id ON email_recipients(email_id);
CREATE INDEX idx_email_recipients_author_id ON email_recipients(author_id);

CREATE TABLE email_recipients_default PARTITION OF email_recipients DEFAULT;

CREATE TABLE email_references (
    mailing_list_id INTEGER NOT NULL,
    email_id INTEGER NOT NULL,
    referenced_message_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    PRIMARY KEY (mailing_list_id, email_id, referenced_message_id)
) PARTITION BY LIST (mailing_list_id);

CREATE INDEX idx_email_references_email_id ON email_references(email_id);
CREATE INDEX idx_email_references_ref_msg_id ON email_references(referenced_message_id);

CREATE TABLE email_references_default PARTITION OF email_references DEFAULT;

CREATE TABLE thread_memberships (
    mailing_list_id INTEGER NOT NULL,
    thread_id INTEGER NOT NULL,
    email_id INTEGER NOT NULL,
    depth INTEGER DEFAULT 0,
    PRIMARY KEY (mailing_list_id, thread_id, email_id)
) PARTITION BY LIST (mailing_list_id);

CREATE INDEX idx_thread_memberships_thread_id ON thread_memberships(thread_id);
CREATE INDEX idx_thread_memberships_email_id ON thread_memberships(email_id);

CREATE TABLE thread_memberships_default PARTITION OF thread_memberships DEFAULT;

-- ============================================================================
-- Auth & notifications
-- ============================================================================
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    auth_provider TEXT NOT NULL CHECK (auth_provider IN ('oidc', 'local', 'hybrid')),
    oidc_sub TEXT,
    oidc_iss TEXT,
    email TEXT NOT NULL UNIQUE,
    display_name TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_login_at TIMESTAMPTZ,
    role TEXT NOT NULL DEFAULT 'user' CHECK (role IN ('admin', 'user')),
    disabled BOOLEAN NOT NULL DEFAULT false,
    token_version INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_users_oidc ON users(oidc_iss, oidc_sub);

CREATE TABLE local_user_credentials (
    user_id INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    password_hash TEXT NOT NULL,
    password_updated_at TIMESTAMPTZ DEFAULT NOW(),
    failed_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until TIMESTAMPTZ,
    mfa_secret BYTEA
);

CREATE TABLE user_profiles (
    user_id INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    preferences JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE user_refresh_tokens (
    token_id UUID PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    hashed_token TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    device_fingerprint TEXT
);

CREATE INDEX user_refresh_tokens_user_idx ON user_refresh_tokens(user_id, expires_at DESC);

CREATE TABLE user_thread_follows (
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    mailing_list_id INTEGER NOT NULL,
    thread_id INTEGER NOT NULL,
    level TEXT NOT NULL DEFAULT 'watch' CHECK (level IN ('watch', 'mute')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, mailing_list_id, thread_id)
);

CREATE TABLE notifications (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    mailing_list_id INTEGER NOT NULL,
    thread_id INTEGER NOT NULL,
    email_id INTEGER,
    type TEXT NOT NULL CHECK (type IN ('new_reply')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    read_at TIMESTAMPTZ
);

CREATE INDEX idx_notifications_user_created ON notifications(user_id, created_at DESC);

CREATE TABLE notification_cursors (
    user_id INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    last_seen_at TIMESTAMPTZ,
    last_seen_notification_id INTEGER REFERENCES notifications(id) ON DELETE SET NULL
);

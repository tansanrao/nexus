-- Initial database schema for Nexus
-- This migration creates all base tables, partitioned tables, and indexes

-- ==============================================================================
-- METADATA TABLES (not partitioned)
-- ==============================================================================

-- Table 0: sync_jobs (job queue with simplified phase tracking)
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

-- Table 1: mailing_lists
CREATE TABLE mailing_lists (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    description TEXT,
    enabled BOOLEAN DEFAULT true,
    sync_priority INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_synced_at TIMESTAMPTZ,
    last_threaded_at TIMESTAMPTZ
);

CREATE INDEX idx_mailing_lists_slug ON mailing_lists(slug);
CREATE INDEX idx_mailing_lists_enabled ON mailing_lists(enabled);

-- Table 2: mailing_list_repositories (supports multiple repos per mailing list)
CREATE TABLE mailing_list_repositories (
    id SERIAL PRIMARY KEY,
    mailing_list_id INTEGER REFERENCES mailing_lists(id) ON DELETE CASCADE,
    repo_url TEXT NOT NULL,
    repo_order INTEGER NOT NULL DEFAULT 0,
    last_indexed_commit TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(mailing_list_id, repo_order)
);

CREATE INDEX idx_mailing_list_repos_list_id ON mailing_list_repositories(mailing_list_id);

-- ==============================================================================
-- GLOBAL TABLES (not partitioned)
-- ==============================================================================

-- Table 3: authors (GLOBAL - not partitioned)
CREATE TABLE authors (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    canonical_name TEXT,
    first_seen TIMESTAMPTZ DEFAULT NOW(),
    last_seen TIMESTAMPTZ DEFAULT NOW()
);

-- Table 3a: author_name_aliases (track name variations)
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

-- Table 3b: author_mailing_list_activity (per-list stats)
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

-- ==============================================================================
-- PARTITIONED TABLES (partitioned by mailing_list_id)
-- ==============================================================================

-- Table 4: emails (partitioned by mailing_list_id)
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
-- Partial index for unthreaded emails (optimizes incremental threading queries)
CREATE INDEX idx_emails_unthreaded ON emails(id) WHERE threaded_at IS NULL;

-- Table 5: threads (partitioned by mailing_list_id)
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

-- Table 6: email_recipients (partitioned by mailing_list_id)
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

-- Table 7: email_references (partitioned by mailing_list_id)
CREATE TABLE email_references (
    mailing_list_id INTEGER NOT NULL,
    email_id INTEGER NOT NULL,
    referenced_message_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    PRIMARY KEY (mailing_list_id, email_id, referenced_message_id)
) PARTITION BY LIST (mailing_list_id);

CREATE INDEX idx_email_references_email_id ON email_references(email_id);
CREATE INDEX idx_email_references_ref_msg_id ON email_references(referenced_message_id);

-- Table 8: thread_memberships (partitioned by mailing_list_id)
CREATE TABLE thread_memberships (
    mailing_list_id INTEGER NOT NULL,
    thread_id INTEGER NOT NULL,
    email_id INTEGER NOT NULL,
    depth INTEGER DEFAULT 0,
    PRIMARY KEY (mailing_list_id, thread_id, email_id)
) PARTITION BY LIST (mailing_list_id);

CREATE INDEX idx_thread_memberships_thread_id ON thread_memberships(thread_id);
CREATE INDEX idx_thread_memberships_email_id ON thread_memberships(email_id);

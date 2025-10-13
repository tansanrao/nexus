// Mailing List types
export interface MailingList {
  id: number;
  name: string;
  slug: string;
  description: string | null;
  enabled: boolean;
  sync_priority: number;
  created_at: string;
  last_synced_at: string | null;
}

export interface MailingListRepository {
  id: number;
  mailing_list_id: number;
  repo_url: string;
  repo_order: number;
  created_at: string;
}

export interface MailingListWithRepos extends MailingList {
  repositories: MailingListRepository[];
}

// Core entity types
export interface Author {
  id: number;
  email: string;
  canonical_name: string | null;
  first_seen: string | null;
  last_seen: string | null;
}

export interface Email {
  id: number;
  mailing_list_id: number;
  message_id: string;
  git_commit_hash: string;
  author_id: number;
  subject: string;
  date: string;
  in_reply_to: string | null;
  body: string | null;
  created_at: string | null;
}

export interface Thread {
  id: number;
  mailing_list_id: number;
  root_message_id: string;
  subject: string;
  start_date: string;
  last_date: string;
  message_count: number | null;
}

export interface EmailWithAuthor extends Email {
  author_name: string | null;
  author_email: string;
}

export interface EmailHierarchy extends EmailWithAuthor {
  depth: number;
}

export interface ThreadDetail {
  thread: Thread;
  emails: EmailHierarchy[];
}

export interface AuthorWithStats extends Author {
  email_count: number;
  thread_count: number;
  first_email_date: string | null;
  last_email_date: string | null;
  mailing_lists: string[];  // Array of mailing list slugs
  name_variations: string[];  // Array of name variations
}

export interface Stats {
  total_emails: number;
  total_threads: number;
  total_authors: number;
  date_range_start: string | null;
  date_range_end: string | null;
}

// Admin types - simplified phase-based tracking
export type JobPhase =
  | 'waiting'
  | 'parsing'
  | 'threading'
  | 'done'
  | 'errored';

export interface JobStatusInfo {
  id: number;
  mailing_list_id: number;
  slug: string;
  name: string;
  phase: JobPhase;
  priority: number;
  created_at: string;
  started_at: string | null;
  completed_at: string | null;
  error_message: string | null;
}

export interface QueuedJob {
  id: number;
  mailing_list_id: number;
  mailing_list_slug: string;
  mailing_list_name: string;
  position: number;
}

export interface GlobalSyncStatus {
  current_job: JobStatusInfo | null;
  queued_jobs: QueuedJob[];
  is_running: boolean;
}

export interface DatabaseStatus {
  total_authors: number;
  total_emails: number;
  total_threads: number;
  total_recipients: number;
  total_references: number;
  total_thread_memberships: number;
  date_range_start: string | null;
  date_range_end: string | null;
}

export interface AdminConfig {
  repo_url: string;
  mirror_path: string;
}

// Query parameter types
export type ThreadSortBy = 'start_date' | 'last_date' | 'message_count';
export type AuthorSortBy = 'canonical_name' | 'email' | 'email_count' | 'thread_count' | 'first_email_date' | 'last_email_date';
export type SortOrder = 'asc' | 'desc';
export type SearchType = 'subject' | 'full_text';

export interface ThreadQueryParams {
  page?: number;
  limit?: number;
  sort_by?: ThreadSortBy;
  order?: SortOrder;
}

export interface ThreadSearchParams {
  search?: string;
  search_type?: SearchType;
  page?: number;
  limit?: number;
  sort_by?: ThreadSortBy;
  order?: SortOrder;
}

export interface AuthorQueryParams {
  search?: string;
  page?: number;
  limit?: number;
  sort_by?: AuthorSortBy;
  order?: SortOrder;
}

// Thread with starter info
export interface ThreadWithStarter extends Thread {
  starter_id: number;
  starter_name: string | null;
  starter_email: string;
}

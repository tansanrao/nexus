export interface MailingList {
  id: number;
  name: string;
  slug: string;
  description: string | null;
  enabled: boolean;
  sync_priority: number;
  created_at: string | null;
  last_synced_at: string | null;
}

export interface MailingListRepository {
  id: number;
  mailing_list_id: number;
  repo_url: string;
  repo_order: number;
  created_at: string;
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

export interface PatchSection {
  start_line: number;
  end_line: number;
}

export interface PatchMetadata {
  diff_sections: PatchSection[];
  diffstat_section: PatchSection | null;
  trailer_sections: PatchSection[];
  separator_line: number | null;
  trailer_count: number;
}

export type PatchType = 'None' | 'Inline' | 'Attachment';

export interface EmailHierarchy {
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
  author_name: string | null;
  author_email: string;
  depth: number;
  patch_type: PatchType;
  is_patch_only: boolean;
  patch_metadata: PatchMetadata | null;
}

export interface ThreadDetail {
  thread: Thread;
  emails: EmailHierarchy[];
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
  author_name: string | null;
  author_email: string;
  patch_type: PatchType;
  is_patch_only: boolean;
  patch_metadata: PatchMetadata | null;
}

export interface Author {
  id: number;
  email: string;
  canonical_name: string | null;
  first_seen: string | null;
  last_seen: string | null;
}

export interface AuthorWithStats extends Author {
  email_count: number;
  thread_count: number;
  first_email_date: string | null;
  last_email_date: string | null;
  mailing_lists: string[];
  name_variations: string[];
}

export interface ThreadWithStarter extends Thread {
  starter_id: number;
  starter_name: string | null;
  starter_email: string;
}

export type SortDirection = 'asc' | 'desc';

export interface SortDescriptor {
  field: string;
  direction: SortDirection;
}

export interface PaginationMeta {
  page: number;
  pageSize: number;
  totalPages: number;
  totalItems: number;
}

export interface ResponseMeta {
  pagination?: PaginationMeta;
  sort?: SortDescriptor[];
  listId?: string;
  filters?: Record<string, unknown>;
  extra?: Record<string, unknown>;
}

export interface ApiEnvelope<T> {
  data: T;
  meta: ResponseMeta;
}

export interface ThreadSearchThreadSummary {
  threadId: number;
  mailingListId: number;
  mailingListSlug: string;
  rootMessageId: string;
  subject: string;
  normalizedSubject?: string | null;
  startDate: string;
  lastActivity: string;
  messageCount: number;
  starterId: number;
  starterName?: string | null;
  starterEmail: string;
}

export interface ThreadSearchParticipant {
  id: number;
  name?: string | null;
  email: string;
}

export interface ThreadSearchScore {
  rankingScore?: number | null;
  semanticRatio: number;
}

export interface ThreadSearchHighlights {
  subjectHtml?: string | null;
  subjectText?: string | null;
  discussionHtml?: string | null;
  discussionText?: string | null;
}

export interface ThreadSearchHit {
  thread: ThreadSearchThreadSummary;
  participants: ThreadSearchParticipant[];
  hasPatches: boolean;
  seriesId?: string | null;
  seriesNumber?: number | null;
  seriesTotal?: number | null;
  firstPostExcerpt?: string | null;
  score: ThreadSearchScore;
  highlights?: ThreadSearchHighlights | null;
}

export interface ThreadSearchPage {
  hits: ThreadSearchHit[];
  total: number;
}

export interface ThreadListItem {
  thread: ThreadWithStarter;
  participants: ThreadSearchParticipant[];
  hasPatches: boolean;
  seriesId?: string | null;
  seriesNumber?: number | null;
  seriesTotal?: number | null;
  firstPostExcerpt?: string | null;
  score: ThreadSearchScore;
  highlights?: ThreadSearchHighlights | null;
}

export interface AuthorSearchMailingListStats {
  slug: string;
  emailCount: number;
  threadCount: number;
  firstEmailDate?: string | null;
  lastEmailDate?: string | null;
}

export interface AuthorSearchHit {
  authorId: number;
  canonicalName?: string | null;
  email: string;
  aliases: string[];
  mailingLists: string[];
  firstSeen?: string | null;
  lastSeen?: string | null;
  firstEmailDate?: string | null;
  lastEmailDate?: string | null;
  threadCount: number;
  emailCount: number;
  mailingListStats: AuthorSearchMailingListStats[];
}

export interface AuthorSearchPage {
  hits: AuthorSearchHit[];
  total: number;
}

export interface MessageResponse {
  message: string;
}

export interface JobEnqueueResponse {
  jobId: number;
  jobType: JobType;
  mailingListId: number | null;
  message: string;
}

export interface PageMetadata {
  page: number;
  size: number;
  totalPages: number;
  totalElements: number;
}

export interface PaginatedResponse<T> {
  data: T[];
  page: PageMetadata;
}

export interface DataResponse<T> {
  data: T;
}

export type JobStatus = 'queued' | 'running' | 'succeeded' | 'failed' | 'cancelled';

export type JobType = 'import' | 'index_maintenance';

export interface JobStatusInfo {
  id: number;
  jobType: JobType | string;
  status: JobStatus;
  priority: number;
  payload?: unknown;
  mailingListId: number | null;
  mailingListSlug: string | null;
  mailingListName: string | null;
  createdAt: string;
  startedAt: string | null;
  completedAt: string | null;
  lastHeartbeat: string | null;
  errorMessage: string | null;
}

export interface QueuedJob {
  id: number;
  mailingListId: number | null;
  mailingListSlug: string | null;
  mailingListName: string | null;
  jobType: JobType;
  status: JobStatus;
  position: number;
}

export interface GlobalSyncStatus {
  currentJob: JobStatusInfo | null;
  queuedJobs: QueuedJob[];
  isRunning: boolean;
}

export interface DatabaseStatus {
  totalAuthors: number;
  totalEmails: number;
  totalThreads: number;
  totalRecipients: number;
  totalReferences: number;
  totalThreadMemberships: number;
  dateRangeStart: string | null;
  dateRangeEnd: string | null;
}

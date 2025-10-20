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

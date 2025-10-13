import type {
  Thread,
  ThreadDetail,
  EmailWithAuthor,
  AuthorWithStats,
  Stats,
  GlobalSyncStatus,
  DatabaseStatus,
  AdminConfig,
  ThreadQueryParams,
  ThreadSearchParams,
  AuthorQueryParams,
  ThreadWithStarter,
  MailingList,
  MailingListRepository,
} from '../types';
import { getApiBaseUrl } from '../contexts/ApiConfigContext';

// Base API call function that supports mailing list context
async function apiCall<T>(
  mailingList: string,
  endpoint: string,
  options?: RequestInit
): Promise<T> {
  const API_BASE_URL = getApiBaseUrl();
  const url = mailingList
    ? `${API_BASE_URL}/${mailingList}${endpoint}`
    : `${API_BASE_URL}${endpoint}`;

  const response = await fetch(url, options);
  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: response.statusText }));
    throw new Error(error.message || `API error: ${response.statusText}`);
  }
  return response.json();
}

// Legacy fetchAPI for backward compatibility with admin endpoints
async function fetchAPI<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const API_BASE_URL = getApiBaseUrl();
  const response = await fetch(`${API_BASE_URL}${endpoint}`, options);
  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: response.statusText }));
    throw new Error(error.message || `API error: ${response.statusText}`);
  }
  return response.json();
}

export const api = {
  // Mailing list endpoints
  mailingLists: {
    list: (): Promise<MailingList[]> =>
      fetchAPI('/admin/mailing-lists'),

    get: (slug: string): Promise<MailingList> =>
      fetchAPI(`/admin/mailing-lists/${slug}`),

    getRepositories: (slug: string): Promise<MailingListRepository[]> =>
      fetchAPI(`/admin/mailing-lists/${slug}/repositories`),

    toggle: (slug: string, enabled: boolean): Promise<{ message: string; enabled: boolean }> =>
      fetchAPI(`/admin/mailing-lists/${slug}/toggle`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ enabled }),
      }),

    seed: (): Promise<{ message: string; mailing_lists_created: number; repositories_created: number; partitions_created: number }> =>
      fetchAPI('/admin/mailing-lists/seed', {
        method: 'POST',
      }),
  },

  // Thread endpoints
  threads: {
    list: (mailingList: string, params: ThreadQueryParams = {}): Promise<Thread[]> => {
      const { page = 1, limit = 50, sort_by, order } = params;
      const sortParam = sort_by ? `&sort_by=${sort_by}` : '';
      const orderParam = order ? `&order=${order}` : '';
      return apiCall(mailingList, `/threads?page=${page}&limit=${limit}${sortParam}${orderParam}`);
    },

    search: (mailingList: string, params: ThreadSearchParams = {}): Promise<Thread[]> => {
      const { search, search_type = 'subject', page = 1, limit = 50, sort_by, order } = params;
      const searchParam = search ? `&search=${encodeURIComponent(search)}` : '';
      const searchTypeParam = `&search_type=${search_type}`;
      const sortParam = sort_by ? `&sort_by=${sort_by}` : '';
      const orderParam = order ? `&order=${order}` : '';
      return apiCall(mailingList, `/threads/search?page=${page}&limit=${limit}${searchParam}${searchTypeParam}${sortParam}${orderParam}`);
    },

    get: (mailingList: string, threadId: number): Promise<ThreadDetail> =>
      apiCall(mailingList, `/threads/${threadId}`),
  },

  // Email endpoints
  emails: {
    get: (mailingList: string, emailId: number): Promise<EmailWithAuthor> =>
      apiCall(mailingList, `/emails/${emailId}`),
  },

  // Author endpoints
  authors: {
    search: (mailingList: string, params: AuthorQueryParams = {}): Promise<AuthorWithStats[]> => {
      const { search, page = 1, limit = 50, sort_by, order } = params;
      const searchParam = search ? `&search=${encodeURIComponent(search)}` : '';
      const sortParam = sort_by ? `&sort_by=${sort_by}` : '';
      const orderParam = order ? `&order=${order}` : '';
      return apiCall(mailingList, `/authors?page=${page}&limit=${limit}${searchParam}${sortParam}${orderParam}`);
    },

    get: (mailingList: string, authorId: number): Promise<AuthorWithStats> =>
      apiCall(mailingList, `/authors/${authorId}`),

    getEmails: (mailingList: string, authorId: number, page: number = 1, limit: number = 50): Promise<EmailWithAuthor[]> =>
      apiCall(mailingList, `/authors/${authorId}/emails?page=${page}&limit=${limit}`),

    getThreadsStarted: (mailingList: string, authorId: number, page: number = 1, limit: number = 50): Promise<ThreadWithStarter[]> =>
      apiCall(mailingList, `/authors/${authorId}/threads-started?page=${page}&limit=${limit}`),

    getThreadsParticipated: (mailingList: string, authorId: number, page: number = 1, limit: number = 50): Promise<Thread[]> =>
      apiCall(mailingList, `/authors/${authorId}/threads-participated?page=${page}&limit=${limit}`),
  },

  // Stats endpoint
  stats: {
    get: (mailingList: string): Promise<Stats> =>
      apiCall(mailingList, '/stats'),
  },

  // Admin endpoints
  admin: {
    sync: {
      queue: (slugs: string[]): Promise<{ job_ids: number[]; message: string }> =>
        fetchAPI('/admin/sync/queue', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ mailing_list_slugs: slugs }),
        }),

      getStatus: (): Promise<GlobalSyncStatus> =>
        fetchAPI('/admin/sync/status'),

      cancel: (): Promise<{ message: string }> =>
        fetchAPI('/admin/sync/cancel', { method: 'POST' }),
    },

    database: {
      reset: (): Promise<{ message: string }> =>
        fetchAPI('/admin/database/reset', { method: 'POST' }),

      getStatus: (): Promise<DatabaseStatus> =>
        fetchAPI('/admin/database/status'),
    },

    config: {
      get: (): Promise<AdminConfig> =>
        fetchAPI('/admin/config'),
    },
  },
};

import type {
  AuthorQueryParams,
  AuthorWithStats,
  DataResponse,
  DatabaseStatus,
  EmailWithAuthor,
  GlobalSyncStatus,
  MailingList,
  MailingListRepository,
  PaginatedResponse,
  Stats,
  Thread,
  ThreadDetail,
  ThreadQueryParams,
  ThreadSearchParams,
  ThreadWithStarter,
  AdminConfig,
} from '../types';
import { getApiBaseUrl } from '../contexts/ApiConfigContext';

type MessageResponse = { message: string };
type ToggleResponse = { message: string; enabled: boolean };
type SyncQueueResponse = { jobIds: number[]; message: string };
type SeedResponse = {
  message: string;
  mailingListsCreated: number;
  repositoriesCreated: number;
  partitionsCreated: number;
};

function normalizeBaseUrl(baseUrl: string): string {
  return baseUrl.replace(/\/+$/, '');
}

function normalizeEndpoint(endpoint: string): string {
  return endpoint.startsWith('/') ? endpoint : `/${endpoint}`;
}

function buildQueryString(params: Record<string, unknown>): string {
  const search = new URLSearchParams();
  Object.entries(params).forEach(([key, value]) => {
    if (value === undefined || value === null || value === '') {
      return;
    }
    search.set(key, String(value));
  });
  const query = search.toString();
  return query ? `?${query}` : '';
}

async function request<T>(url: string, options?: RequestInit): Promise<T> {
  const headers = new Headers(options?.headers);
  if (!headers.has('Accept')) {
    headers.set('Accept', 'application/json');
  }

  const response = await fetch(url, {
    ...options,
    headers,
  });

  if (!response.ok) {
    let message = response.statusText;
    try {
      const errorBody = await response.json();
      if (errorBody && typeof errorBody === 'object' && 'message' in errorBody) {
        message = String(errorBody.message);
      }
    } catch {
      // Ignore JSON parsing failures for error responses
    }
    throw new Error(message || `API error: ${response.statusText}`);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return response.json() as Promise<T>;
}

async function apiCall<T>(mailingList: string, endpoint: string, options?: RequestInit): Promise<T> {
  const baseUrl = normalizeBaseUrl(getApiBaseUrl());
  const path = normalizeEndpoint(endpoint);
  const slug = encodeURIComponent(mailingList);
  return request<T>(`${baseUrl}/${slug}${path}`, options);
}

async function fetchAPI<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const baseUrl = normalizeBaseUrl(getApiBaseUrl());
  const path = normalizeEndpoint(endpoint);
  return request<T>(`${baseUrl}${path}`, options);
}

export const api = {
  mailingLists: {
    list: async (): Promise<MailingList[]> => {
      const response = await fetchAPI<DataResponse<MailingList[]>>('/admin/mailing-lists');
      return response.data;
    },

    get: (slug: string): Promise<MailingList> =>
      fetchAPI(`/admin/mailing-lists/${encodeURIComponent(slug)}`),

    getRepositories: (slug: string): Promise<MailingListRepository[]> =>
      fetchAPI(`/admin/mailing-lists/${encodeURIComponent(slug)}/repositories`),

    toggle: (slug: string, enabled: boolean): Promise<ToggleResponse> =>
      fetchAPI(`/admin/mailing-lists/${encodeURIComponent(slug)}/toggle`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ enabled }),
      }),

    seed: (): Promise<SeedResponse> =>
      fetchAPI('/admin/mailing-lists/seed', {
        method: 'POST',
      }),
  },

  threads: {
    list: async (mailingList: string, params: ThreadQueryParams = {}): Promise<PaginatedResponse<Thread>> => {
      const { page = 1, size = 50, sortBy, order } = params;
      const query = buildQueryString({ page, size, sortBy, order });
      return apiCall<PaginatedResponse<Thread>>(mailingList, `/threads${query}`);
    },

    search: async (mailingList: string, params: ThreadSearchParams = {}): Promise<PaginatedResponse<Thread>> => {
      const {
        q,
        searchType = 'subject',
        page = 1,
        size = 50,
        sortBy,
        order,
      } = params;
      const query = buildQueryString({ q, searchType, page, size, sortBy, order });
      return apiCall<PaginatedResponse<Thread>>(mailingList, `/threads/search${query}`);
    },

    get: (mailingList: string, threadId: number): Promise<ThreadDetail> =>
      apiCall(mailingList, `/threads/${threadId}`),
  },

  emails: {
    get: (mailingList: string, emailId: number): Promise<EmailWithAuthor> =>
      apiCall(mailingList, `/emails/${emailId}`),
  },

  authors: {
    search: async (mailingList: string, params: AuthorQueryParams = {}): Promise<PaginatedResponse<AuthorWithStats>> => {
      const { q, page = 1, size = 50, sortBy, order } = params;
      const query = buildQueryString({ q, page, size, sortBy, order });
      return apiCall<PaginatedResponse<AuthorWithStats>>(mailingList, `/authors${query}`);
    },

    get: (mailingList: string, authorId: number): Promise<AuthorWithStats> =>
      apiCall(mailingList, `/authors/${authorId}`),

    getEmails: async (
      mailingList: string,
      authorId: number,
      page: number = 1,
      size: number = 50,
    ): Promise<PaginatedResponse<EmailWithAuthor>> => {
      const query = buildQueryString({ page, size });
      return apiCall<PaginatedResponse<EmailWithAuthor>>(mailingList, `/authors/${authorId}/emails${query}`);
    },

    getThreadsStarted: async (
      mailingList: string,
      authorId: number,
      page: number = 1,
      size: number = 50,
    ): Promise<PaginatedResponse<ThreadWithStarter>> => {
      const query = buildQueryString({ page, size });
      return apiCall<PaginatedResponse<ThreadWithStarter>>(mailingList, `/authors/${authorId}/threads-started${query}`);
    },

    getThreadsParticipated: async (
      mailingList: string,
      authorId: number,
      page: number = 1,
      size: number = 50,
    ): Promise<PaginatedResponse<Thread>> => {
      const query = buildQueryString({ page, size });
      return apiCall<PaginatedResponse<Thread>>(mailingList, `/authors/${authorId}/threads-participated${query}`);
    },
  },

  stats: {
    get: (mailingList: string): Promise<Stats> =>
      apiCall(mailingList, '/stats'),
  },

  admin: {
    sync: {
      queue: (slugs: string[]): Promise<SyncQueueResponse> =>
        fetchAPI('/admin/sync/queue', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ mailingListSlugs: slugs }),
        }),

      getStatus: (): Promise<GlobalSyncStatus> =>
        fetchAPI('/admin/sync/status'),

      start: (): Promise<MessageResponse> =>
        fetchAPI('/admin/sync/start', { method: 'POST' }),

      cancel: (): Promise<MessageResponse> =>
        fetchAPI('/admin/sync/cancel', { method: 'POST' }),
    },

    database: {
      reset: (): Promise<MessageResponse> =>
        fetchAPI('/admin/database/reset', { method: 'POST' }),

      getStatus: (): Promise<DatabaseStatus> =>
        fetchAPI('/admin/database/status'),

      getConfig: (): Promise<AdminConfig> =>
        fetchAPI('/admin/database/config'),
    },
  },
};

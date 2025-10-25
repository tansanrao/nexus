import type {
  MailingList,
  MailingListRepository,
  ThreadDetail,
  Email,
  AuthorWithStats,
  ThreadWithStarter,
  PaginatedResponse,
  DataResponse,
  GlobalSyncStatus,
  DatabaseStatus,
  ThreadSearchResponse,
  MessageResponse,
  JobEnqueueResponse,
} from '../types';
import { getApiBaseUrl } from '../contexts/ApiConfigContext';

type ThreadSortField = 'startDate' | 'lastDate' | 'messageCount';
type SortOrder = 'asc' | 'desc';

interface AuthorThreadQueryParams {
  page?: number;
  size?: number;
  sortBy?: ThreadSortField;
  order?: SortOrder;
  query?: string;
}

const API_PREFIX = '/api/v1';

interface ToggleResponse {
  message: string;
  enabled: boolean;
}

interface SyncQueueResponse {
  jobIds: number[];
  message: string;
}

interface SeedResponse {
  message: string;
  mailingListsCreated: number;
  repositoriesCreated: number;
  partitionsCreated: number;
}

interface SearchIndexRefreshParams {
  mailingListSlug?: string;
  reindex?: boolean;
}

interface IndexMaintenanceParams {
  mailingListSlug?: string;
  reindex?: boolean;
}

export class ApiClient {
  private getNormalizedBaseUrl(): string {
    let baseUrl = getApiBaseUrl().trim();
    baseUrl = baseUrl.replace(/\/+$/, '');

    if (/\/api(?:\/v1)?$/i.test(baseUrl)) {
      baseUrl = baseUrl.replace(/\/api(?:\/v1)?$/i, '');
    }

    return baseUrl;
  }

  private async request<T>(path: string, init: RequestInit = {}): Promise<T> {
    const baseUrl = this.getNormalizedBaseUrl();
    const headers = new Headers(init.headers);
    if (!headers.has('Accept')) {
      headers.set('Accept', 'application/json');
    }
    if (init.body && !headers.has('Content-Type')) {
      headers.set('Content-Type', 'application/json');
    }

    const response = await fetch(`${baseUrl}${path}`, {
      ...init,
      headers,
    });

    if (!response.ok) {
      let message = `${response.status} ${response.statusText}`;
      try {
        const errorBody = await response.json();
        if (errorBody && typeof errorBody === 'object' && 'message' in errorBody) {
          message = String(errorBody.message);
        }
      } catch {
        // ignore JSON parse errors
      }
      throw new Error(`API error: ${message}`);
    }

    if (response.status === 204) {
      return undefined as T;
    }

    return response.json() as Promise<T>;
  }

  private buildAuthorThreadQuery(params: AuthorThreadQueryParams): string {
    const searchParams = new URLSearchParams();

    if (typeof params.page === 'number') {
      searchParams.set('page', params.page.toString());
    }
    if (typeof params.size === 'number') {
      searchParams.set('size', params.size.toString());
    }
    if (params.sortBy) {
      searchParams.set('sortBy', params.sortBy);
    }
    if (params.order) {
      searchParams.set('order', params.order);
    }
    if (params.query && params.query.trim()) {
      searchParams.set('q', params.query.trim());
    }

    return searchParams.toString();
  }

  async getMailingLists(): Promise<MailingList[]> {
    const result = await this.request<DataResponse<MailingList[]>>(
      `${API_PREFIX}/admin/mailing-lists`
    );
    return result.data;
  }

  async getMailingList(slug: string): Promise<MailingList> {
    return this.request<MailingList>(`${API_PREFIX}/admin/mailing-lists/${slug}`);
  }

  async getMailingListRepositories(slug: string): Promise<MailingListRepository[]> {
    return this.request<MailingListRepository[]>(
      `${API_PREFIX}/admin/mailing-lists/${slug}/repositories`
    );
  }

  async toggleMailingList(slug: string, enabled: boolean): Promise<ToggleResponse> {
    return this.request<ToggleResponse>(`${API_PREFIX}/admin/mailing-lists/${slug}/toggle`, {
      method: 'PATCH',
      body: JSON.stringify({ enabled }),
    });
  }

  async seedMailingLists(): Promise<SeedResponse> {
    return this.request<SeedResponse>(`${API_PREFIX}/admin/mailing-lists/seed`, {
      method: 'POST',
    });
  }

  async refreshSearchIndex(params: SearchIndexRefreshParams = {}): Promise<JobEnqueueResponse> {
    const payload: Record<string, unknown> = {};
    if (params.mailingListSlug && params.mailingListSlug.trim()) {
      payload.mailingListSlug = params.mailingListSlug.trim();
    }
    if (typeof params.reindex === 'boolean') {
      payload.reindex = params.reindex;
    }

    return this.request<JobEnqueueResponse>(`${API_PREFIX}/admin/search/index/refresh`, {
      method: 'POST',
      body: JSON.stringify(payload),
    });
  }

  async resetSearchIndexes(params: IndexMaintenanceParams = {}): Promise<JobEnqueueResponse> {
    const payload: Record<string, unknown> = {};
    if (params.mailingListSlug && params.mailingListSlug.trim()) {
      payload.mailingListSlug = params.mailingListSlug.trim();
    }
    if (typeof params.reindex === 'boolean') {
      payload.reindex = params.reindex;
    }

    return this.request<JobEnqueueResponse>(`${API_PREFIX}/admin/search/index/reset`, {
      method: 'POST',
      body: JSON.stringify(payload),
    });
  }

  async getThreads(
    slug: string,
    page: number = 1,
    size: number = 50,
    sortBy: ThreadSortField = 'lastDate',
    order: SortOrder = 'desc'
  ): Promise<PaginatedResponse<ThreadWithStarter>> {
    const params = new URLSearchParams({
      page: page.toString(),
      size: size.toString(),
      sortBy,
      order,
    });

    return this.request<PaginatedResponse<ThreadWithStarter>>(
      `${API_PREFIX}/${slug}/threads?${params.toString()}`
    );
  }

  async searchThreads(
    slug: string,
    query: string,
    page: number = 1,
    size: number = 25
  ): Promise<ThreadSearchResponse> {
    const params = new URLSearchParams({
      page: page.toString(),
      size: size.toString(),
    });

    if (query.trim()) {
      params.set('q', query.trim());
    }

    return this.request<ThreadSearchResponse>(
      `${API_PREFIX}/${slug}/threads/search?${params.toString()}`
    );
  }

  async getThread(slug: string, threadId: number): Promise<ThreadDetail> {
    return this.request<ThreadDetail>(`${API_PREFIX}/${slug}/threads/${threadId}`);
  }

  async getEmail(slug: string, emailId: number): Promise<Email> {
    return this.request<Email>(`${API_PREFIX}/${slug}/emails/${emailId}`);
  }

  async searchAuthors(
    slug: string,
    query: string,
    page: number = 1,
    size: number = 50,
    sortBy?: string,
    order?: SortOrder
  ): Promise<PaginatedResponse<AuthorWithStats>> {
    const params = new URLSearchParams({
      page: page.toString(),
      size: size.toString(),
    });

    if (query.trim()) {
      params.set('q', query.trim());
    }
    if (sortBy) {
      params.set('sortBy', sortBy);
    }
    if (order) {
      params.set('order', order);
    }

    return this.request<PaginatedResponse<AuthorWithStats>>(
      `${API_PREFIX}/${slug}/authors?${params.toString()}`
    );
  }

  async getAuthor(slug: string, authorId: number): Promise<AuthorWithStats> {
    return this.request<AuthorWithStats>(`${API_PREFIX}/${slug}/authors/${authorId}`);
  }

  async getAuthorThreadsStarted(
    slug: string,
    authorId: number,
    params: AuthorThreadQueryParams = {}
  ): Promise<PaginatedResponse<ThreadWithStarter>> {
    const query = this.buildAuthorThreadQuery(params);
    const suffix = query ? `?${query}` : '';

    return this.request<PaginatedResponse<ThreadWithStarter>>(
      `${API_PREFIX}/${slug}/authors/${authorId}/threads-started${suffix}`
    );
  }

  async getAuthorThreadsParticipated(
    slug: string,
    authorId: number,
    params: AuthorThreadQueryParams = {}
  ): Promise<PaginatedResponse<ThreadWithStarter>> {
    const query = this.buildAuthorThreadQuery(params);
    const suffix = query ? `?${query}` : '';

    return this.request<PaginatedResponse<ThreadWithStarter>>(
      `${API_PREFIX}/${slug}/authors/${authorId}/threads-participated${suffix}`
    );
  }

  async getSyncStatus(): Promise<GlobalSyncStatus> {
    return this.request<GlobalSyncStatus>(`${API_PREFIX}/admin/sync/status`);
  }

  async queueSync(slugs: string[]): Promise<SyncQueueResponse> {
    return this.request<SyncQueueResponse>(`${API_PREFIX}/admin/sync/queue`, {
      method: 'POST',
      body: JSON.stringify({ mailingListSlugs: slugs }),
    });
  }

  async cancelSync(): Promise<MessageResponse> {
    return this.request<MessageResponse>(`${API_PREFIX}/admin/sync/cancel`, {
      method: 'POST',
    });
  }

  async resetDatabase(): Promise<MessageResponse> {
    return this.request<MessageResponse>(`${API_PREFIX}/admin/database/reset`, {
      method: 'POST',
    });
  }

  async getDatabaseStatus(): Promise<DatabaseStatus> {
    return this.request<DatabaseStatus>(`${API_PREFIX}/admin/database/status`);
  }

  async testConnection(): Promise<boolean> {
    try {
      await this.getMailingLists();
      return true;
    } catch {
      return false;
    }
  }
}

export const apiClient = new ApiClient();

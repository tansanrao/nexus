import type {
  MailingList,
  MailingListRepository,
  ThreadDetail,
  Email,
  AuthorWithStats,
  ThreadWithStarter,
  PaginatedResponse,
  GlobalSyncStatus,
  DatabaseStatus,
  ApiEnvelope,
  ThreadSearchPage,
  AuthorSearchPage,
  MessageResponse,
  JobEnqueueResponse,
  JobStatusInfo,
  JobType,
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
const ADMIN_PREFIX = '/admin/v1';

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
}

interface IndexMaintenanceParams {
  mailingListSlug?: string;
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
    const result = await this.request<ApiEnvelope<MailingList[]>>(
      `${ADMIN_PREFIX}/mailing-lists`
    );
    return result.data;
  }

  async getMailingList(slug: string): Promise<MailingList> {
    const result = await this.request<ApiEnvelope<MailingList>>(
      `${ADMIN_PREFIX}/mailing-lists/${slug}`
    );
    return result.data;
  }

  async getMailingListRepositories(slug: string): Promise<MailingListRepository[]> {
    const result = await this.request<ApiEnvelope<MailingListRepository[]>>(
      `${ADMIN_PREFIX}/mailing-lists/${slug}/repositories`
    );
    return result.data;
  }

  async toggleMailingList(slug: string, enabled: boolean): Promise<ToggleResponse> {
    return this.request<ToggleResponse>(`${ADMIN_PREFIX}/mailing-lists/${slug}/toggle`, {
      method: 'PATCH',
      body: JSON.stringify({ enabled }),
    });
  }

  async seedMailingLists(): Promise<SeedResponse> {
    return this.request<SeedResponse>(`${ADMIN_PREFIX}/mailing-lists/seed`, {
      method: 'POST',
    });
  }

  async refreshSearchIndex(params: SearchIndexRefreshParams = {}): Promise<JobEnqueueResponse> {
    const payload: Record<string, unknown> = {};
    if (params.mailingListSlug && params.mailingListSlug.trim()) {
      payload.mailingListSlug = params.mailingListSlug.trim();
    }

    const response = await this.request<ApiEnvelope<JobStatusInfo>>(
      `${ADMIN_PREFIX}/search/indexes/threads/refresh`,
      {
        method: 'POST',
        body: JSON.stringify(payload),
      }
    );

    const job = response.data;
    const jobType =
      job.jobType === 'import' || job.jobType === 'index_maintenance'
        ? (job.jobType as JobType)
        : 'index_maintenance';
    return {
      jobId: job.id,
      jobType,
      mailingListId: job.mailingListId ?? null,
      message: `Job ${job.id} queued`,
    };
  }

  async resetSearchIndexes(params: IndexMaintenanceParams = {}): Promise<JobEnqueueResponse> {
    const payload: Record<string, unknown> = {};
    if (params.mailingListSlug && params.mailingListSlug.trim()) {
      payload.mailingListSlug = params.mailingListSlug.trim();
    }

    const response = await this.request<ApiEnvelope<JobStatusInfo>>(
      `${ADMIN_PREFIX}/search/indexes/reset`,
      {
        method: 'POST',
        body: JSON.stringify(payload),
      }
    );

    const job = response.data;
    const jobType =
      job.jobType === 'import' || job.jobType === 'index_maintenance'
        ? (job.jobType as JobType)
        : 'index_maintenance';
    return {
      jobId: job.id,
      jobType,
      mailingListId: job.mailingListId ?? null,
      message: `Job ${job.id} queued`,
    };
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
    size: number = 25,
    semanticRatio?: number
  ): Promise<ApiEnvelope<ThreadSearchPage>> {
    const params = new URLSearchParams({
      page: page.toString(),
      size: size.toString(),
    });

    if (query.trim()) {
      params.set('q', query.trim());
    }
    if (typeof semanticRatio === 'number') {
      const clamped = Math.max(0, Math.min(1, semanticRatio));
      params.set('semanticRatio', clamped.toFixed(2));
    }

    return this.request<ApiEnvelope<ThreadSearchPage>>(
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
    params.append('mailingList', slug);

    const response = await this.request<ApiEnvelope<AuthorSearchPage>>(
      `${API_PREFIX}/authors/search?${params.toString()}`
    );

    const pagination = response.meta.pagination;
    const hits = response.data.hits.map<AuthorWithStats>((hit) => ({
      id: hit.authorId,
      email: hit.email,
      canonical_name: hit.canonicalName ?? null,
      first_seen: hit.firstSeen ?? null,
      last_seen: hit.lastSeen ?? null,
      email_count: hit.emailCount,
      thread_count: hit.threadCount,
      first_email_date: hit.firstEmailDate ?? null,
      last_email_date: hit.lastEmailDate ?? null,
      mailing_lists: hit.mailingLists,
      name_variations: hit.aliases,
    }));

    return {
      data: hits,
      page: {
        page: pagination?.page ?? page,
        size: pagination?.pageSize ?? size,
        totalPages:
          pagination?.totalPages ??
          (response.data.total > 0 ? Math.ceil(response.data.total / size) : 0),
        totalElements: pagination?.totalItems ?? response.data.total,
      },
    };
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
    return this.request<GlobalSyncStatus>(`${ADMIN_PREFIX}/sync/status`);
  }

  async queueSync(slugs: string[]): Promise<SyncQueueResponse> {
    return this.request<SyncQueueResponse>(`${ADMIN_PREFIX}/sync/queue`, {
      method: 'POST',
      body: JSON.stringify({ mailingListSlugs: slugs }),
    });
  }

  async cancelSync(): Promise<MessageResponse> {
    return this.request<MessageResponse>(`${ADMIN_PREFIX}/sync/cancel`, {
      method: 'POST',
    });
  }

  async resetDatabase(): Promise<MessageResponse> {
    return this.request<MessageResponse>(`${ADMIN_PREFIX}/database/reset`, {
      method: 'POST',
    });
  }

  async getDatabaseStatus(): Promise<DatabaseStatus> {
    return this.request<DatabaseStatus>(`${ADMIN_PREFIX}/database/status`);
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

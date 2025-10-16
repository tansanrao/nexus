import type {
  MailingList,
  Thread,
  ThreadDetail,
  Email,
  AuthorWithStats,
  ThreadWithStarter,
  PaginatedResponse,
  DataResponse,
} from '../types';
import { getApiBaseUrl } from '../contexts/ApiConfigContext';

type ThreadSortField = 'startDate' | 'lastDate' | 'messageCount';
type SortOrder = 'asc' | 'desc';
type ThreadSearchType = 'subject' | 'fullText';

interface AuthorThreadQueryParams {
  page?: number;
  size?: number;
  sortBy?: ThreadSortField;
  order?: SortOrder;
  searchType?: ThreadSearchType;
  query?: string;
}

const API_PREFIX = '/api/v1';

export class ApiClient {
  private getNormalizedBaseUrl(): string {
    let baseUrl = getApiBaseUrl().trim();
    baseUrl = baseUrl.replace(/\/+$/, '');

    if (/\/api(?:\/v1)?$/i.test(baseUrl)) {
      baseUrl = baseUrl.replace(/\/api(?:\/v1)?$/i, '');
    }

    return baseUrl;
  }

  private async fetchJson<T>(path: string): Promise<T> {
    const baseUrl = this.getNormalizedBaseUrl();
    const response = await fetch(`${baseUrl}${path}`, {
      headers: {
        Accept: 'application/json',
      },
    });

    if (!response.ok) {
      throw new Error(`API error: ${response.status} ${response.statusText}`);
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
    if (params.searchType) {
      searchParams.set('searchType', params.searchType);
    }
    if (params.query && params.query.trim()) {
      searchParams.set('q', params.query.trim());
    }

    return searchParams.toString();
  }

  async getMailingLists(): Promise<MailingList[]> {
    const result = await this.fetchJson<DataResponse<MailingList[]>>(
      `${API_PREFIX}/admin/mailing-lists`
    );
    return result.data;
  }

  async getThreads(
    slug: string,
    page: number = 1,
    size: number = 50,
    sortBy: ThreadSortField = 'lastDate',
    order: SortOrder = 'desc'
  ): Promise<PaginatedResponse<Thread>> {
    const params = new URLSearchParams({
      page: page.toString(),
      size: size.toString(),
      sortBy,
      order,
    });

    return this.fetchJson<PaginatedResponse<Thread>>(
      `${API_PREFIX}/${slug}/threads?${params.toString()}`
    );
  }

  async searchThreads(
    slug: string,
    query: string,
    searchType: ThreadSearchType = 'subject',
    page: number = 1,
    size: number = 50,
    sortBy: ThreadSortField = 'lastDate',
    order: SortOrder = 'desc'
  ): Promise<PaginatedResponse<Thread>> {
    const params = new URLSearchParams({
      page: page.toString(),
      size: size.toString(),
      sortBy,
      order,
    });

    if (query.trim()) {
      params.set('q', query.trim());
    }

    params.set('searchType', searchType);

    return this.fetchJson<PaginatedResponse<Thread>>(
      `${API_PREFIX}/${slug}/threads/search?${params.toString()}`
    );
  }

  async getThread(slug: string, threadId: number): Promise<ThreadDetail> {
    return this.fetchJson<ThreadDetail>(`${API_PREFIX}/${slug}/threads/${threadId}`);
  }

  async getEmail(slug: string, emailId: number): Promise<Email> {
    return this.fetchJson<Email>(`${API_PREFIX}/${slug}/emails/${emailId}`);
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

    return this.fetchJson<PaginatedResponse<AuthorWithStats>>(
      `${API_PREFIX}/${slug}/authors?${params.toString()}`
    );
  }

  async getAuthor(slug: string, authorId: number): Promise<AuthorWithStats> {
    return this.fetchJson<AuthorWithStats>(`${API_PREFIX}/${slug}/authors/${authorId}`);
  }

  async getAuthorThreadsStarted(
    slug: string,
    authorId: number,
    params: AuthorThreadQueryParams = {}
  ): Promise<PaginatedResponse<ThreadWithStarter>> {
    const query = this.buildAuthorThreadQuery(params);
    const suffix = query ? `?${query}` : '';

    return this.fetchJson<PaginatedResponse<ThreadWithStarter>>(
      `${API_PREFIX}/${slug}/authors/${authorId}/threads-started${suffix}`
    );
  }

  async getAuthorThreadsParticipated(
    slug: string,
    authorId: number,
    params: AuthorThreadQueryParams = {}
  ): Promise<PaginatedResponse<Thread>> {
    const query = this.buildAuthorThreadQuery(params);
    const suffix = query ? `?${query}` : '';

    return this.fetchJson<PaginatedResponse<Thread>>(
      `${API_PREFIX}/${slug}/authors/${authorId}/threads-participated${suffix}`
    );
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

import type {
  MailingList,
  Thread,
  ThreadDetail,
  Email,
  AuthorWithStats,
  ThreadWithStarter,
} from '../types';
import { getApiBaseUrl } from '../contexts/ApiConfigContext';

export class ApiClient {
  private async fetchJson<T>(path: string): Promise<T> {
    let baseUrl = getApiBaseUrl();
    
    // Remove trailing slash from baseUrl if present
    baseUrl = baseUrl.replace(/\/$/, '');
    
    // If baseUrl ends with /api, remove it since our paths already include /api
    // This handles migration from old frontend which stored URLs with /api
    if (baseUrl.endsWith('/api')) {
      baseUrl = baseUrl.slice(0, -4);
    }
    
    const response = await fetch(`${baseUrl}${path}`);
    if (!response.ok) {
      throw new Error(`API error: ${response.statusText}`);
    }
    return response.json();
  }

  private async fetchJsonWithHeaders<T>(path: string): Promise<{ data: T; headers: Headers }> {
    let baseUrl = getApiBaseUrl();
    baseUrl = baseUrl.replace(/\/$/, '');
    if (baseUrl.endsWith('/api')) {
      baseUrl = baseUrl.slice(0, -4);
    }

    const response = await fetch(`${baseUrl}${path}`);
    if (!response.ok) {
      throw new Error(`API error: ${response.statusText}`);
    }
    const data = await response.json();
    return { data, headers: response.headers };
  }

  async getMailingLists(): Promise<MailingList[]> {
    return this.fetchJson<MailingList[]>('/api/admin/mailing-lists');
  }

  async getThreads(
    slug: string,
    page: number = 1,
    limit: number = 50,
    sortBy: 'start_date' | 'last_date' | 'message_count' = 'last_date',
    order: 'asc' | 'desc' = 'desc'
  ): Promise<Thread[]> {
    const params = new URLSearchParams({
      page: page.toString(),
      limit: limit.toString(),
      sort_by: sortBy,
      order,
    });
    return this.fetchJson<Thread[]>(`/api/${slug}/threads?${params}`);
  }

  async getThreadsWithTotal(
    slug: string,
    page: number = 1,
    limit: number = 50,
    sortBy: 'start_date' | 'last_date' | 'message_count' = 'last_date',
    order: 'asc' | 'desc' = 'desc'
  ): Promise<{ items: Thread[]; total: number | null }> {
    const params = new URLSearchParams({
      page: page.toString(),
      limit: limit.toString(),
      sort_by: sortBy,
      order,
    });
    const { data, headers } = await this.fetchJsonWithHeaders<Thread[]>(`/api/${slug}/threads?${params}`);
    // Try common header names and Content-Range fallback
    let total: number | null = null;
    const totalHeader = headers.get('X-Total-Count') || headers.get('X-Total') || headers.get('Total-Count');
    if (totalHeader) {
      const parsed = parseInt(totalHeader, 10);
      total = Number.isFinite(parsed) ? parsed : null;
    }
    if (total == null) {
      const contentRange = headers.get('Content-Range'); // e.g., "items 0-49/1234"
      if (contentRange) {
        const match = contentRange.match(/\/(\d+)$/);
        if (match && match[1]) {
          const parsed = parseInt(match[1], 10);
          total = Number.isFinite(parsed) ? parsed : null;
        }
      }
    }
    return { items: data, total: Number.isFinite(total) ? (total as number) : null };
  }

  async searchThreads(
    slug: string,
    search: string,
    searchType: 'subject' | 'full_text' = 'subject',
    page: number = 1,
    limit: number = 50,
    sortBy: 'start_date' | 'last_date' | 'message_count' = 'last_date',
    order: 'asc' | 'desc' = 'desc'
  ): Promise<Thread[]> {
    const params = new URLSearchParams({
      search,
      search_type: searchType,
      page: page.toString(),
      limit: limit.toString(),
      sort_by: sortBy,
      order,
    });
    return this.fetchJson<Thread[]>(`/api/${slug}/threads/search?${params}`);
  }

  async searchThreadsWithTotal(
    slug: string,
    search: string,
    searchType: 'subject' | 'full_text' = 'subject',
    page: number = 1,
    limit: number = 50,
    sortBy: 'start_date' | 'last_date' | 'message_count' = 'last_date',
    order: 'asc' | 'desc' = 'desc'
  ): Promise<{ items: Thread[]; total: number | null }> {
    const params = new URLSearchParams({
      search,
      search_type: searchType,
      page: page.toString(),
      limit: limit.toString(),
      sort_by: sortBy,
      order,
    });
    const { data, headers } = await this.fetchJsonWithHeaders<Thread[]>(`/api/${slug}/threads/search?${params}`);
    let total: number | null = null;
    const totalHeader = headers.get('X-Total-Count') || headers.get('X-Total') || headers.get('Total-Count');
    if (totalHeader) {
      const parsed = parseInt(totalHeader, 10);
      total = Number.isFinite(parsed) ? parsed : null;
    }
    if (total == null) {
      const contentRange = headers.get('Content-Range');
      if (contentRange) {
        const match = contentRange.match(/\/(\d+)$/);
        if (match && match[1]) {
          const parsed = parseInt(match[1], 10);
          total = Number.isFinite(parsed) ? parsed : null;
        }
      }
    }
    return { items: data, total: Number.isFinite(total) ? (total as number) : null };
  }

  async getThread(slug: string, threadId: number): Promise<ThreadDetail> {
    return this.fetchJson<ThreadDetail>(`/api/${slug}/threads/${threadId}`);
  }

  async getEmail(slug: string, emailId: number): Promise<Email> {
    return this.fetchJson<Email>(`/api/${slug}/emails/${emailId}`);
  }

  async searchAuthors(slug: string, search: string): Promise<AuthorWithStats[]> {
    const params = new URLSearchParams({ search });
    return this.fetchJson<AuthorWithStats[]>(`/api/${slug}/authors/search?${params}`);
  }

  async getAuthor(slug: string, authorId: number): Promise<AuthorWithStats> {
    return this.fetchJson<AuthorWithStats>(`/api/${slug}/authors/${authorId}`);
  }

  async getAuthorThreadsStarted(
    slug: string,
    authorId: number,
    page: number = 1,
    limit: number = 50
  ): Promise<ThreadWithStarter[]> {
    const params = new URLSearchParams({
      page: page.toString(),
      limit: limit.toString(),
    });
    return this.fetchJson<ThreadWithStarter[]>(
      `/api/${slug}/authors/${authorId}/threads-started?${params}`
    );
  }

  async getAuthorThreadsParticipated(
    slug: string,
    authorId: number,
    page: number = 1,
    limit: number = 50
  ): Promise<Thread[]> {
    const params = new URLSearchParams({
      page: page.toString(),
      limit: limit.toString(),
    });
    return this.fetchJson<Thread[]>(
      `/api/${slug}/authors/${authorId}/threads-participated?${params}`
    );
  }

  async testConnection(): Promise<boolean> {
    try {
      await this.fetchJson('/api/admin/mailing-lists');
      return true;
    } catch {
      return false;
    }
  }
}

// Default API client instance
export const apiClient = new ApiClient();


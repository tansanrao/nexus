import type {
  AuthorListParams,
  JobListParams,
  PaginationParams,
  ThreadListParams,
  ThreadSearchParams,
} from "./types"

const serialize = (value: unknown) => (value ? JSON.stringify(value) : undefined)

export const queryKeys = {
  health: () => ["health"] as const,
  mailingLists: {
    all: () => ["mailingLists"] as const,
    detail: (slug: string) => ["mailingLists", slug] as const,
    repositories: (slug: string) => ["mailingLists", slug, "repositories"] as const,
    withRepos: (slug: string) => ["mailingLists", slug, "withRepos"] as const,
  },
  threads: {
    list: (slug: string, params?: ThreadListParams) =>
      ["threads", slug, "list", serialize(params)] as const,
    search: (slug: string, params: ThreadSearchParams) =>
      ["threads", slug, "search", serialize(params)] as const,
    detail: (slug: string, threadId: string) => ["threads", slug, threadId] as const,
  },
  authors: {
    search: (slug: string, params?: AuthorListParams) =>
      ["authors", slug, "search", serialize(params)] as const,
    detail: (slug: string, authorId: number) => ["authors", slug, authorId] as const,
    emails: (slug: string, authorId: number, params?: PaginationParams) =>
      ["authors", slug, authorId, "emails", serialize(params)] as const,
    threadsStarted: (slug: string, authorId: number, params?: PaginationParams) =>
      ["authors", slug, authorId, "threads-started", serialize(params)] as const,
    threadsParticipated: (slug: string, authorId: number, params?: PaginationParams) =>
      ["authors", slug, authorId, "threads-participated", serialize(params)] as const,
  },
  emails: {
    detail: (slug: string, emailId: number) => ["emails", slug, emailId] as const,
  },
  stats: {
    summary: (slug: string) => ["stats", slug] as const,
  },
  admin: {
    databaseStatus: () => ["admin", "database-status"] as const,
    databaseConfig: () => ["admin", "database-config"] as const,
    jobs: (params?: JobListParams) => ["admin", "jobs", serialize(params)] as const,
    job: (jobId: number) => ["admin", "jobs", jobId] as const,
  },
} as const

import { getJson } from "./http"
import type {
  AuthorSearchParams,
  AuthorWithStats,
  EmailWithAuthor,
  PaginatedResponse,
  PaginationParams,
  ThreadWithStarter,
} from "./types"

export async function searchAuthors(
  slug: string,
  params?: AuthorSearchParams
): Promise<PaginatedResponse<AuthorWithStats[]>> {
  return getJson<PaginatedResponse<AuthorWithStats[]>>(`${slug}/authors`, {
    searchParams: params ? { params } : undefined,
  })
}

export async function getAuthor(slug: string, authorId: number): Promise<AuthorWithStats> {
  return getJson<AuthorWithStats>(`${slug}/authors/${authorId}`)
}

export async function getAuthorEmails(
  slug: string,
  authorId: number,
  params?: PaginationParams
): Promise<PaginatedResponse<EmailWithAuthor[]>> {
  return getJson<PaginatedResponse<EmailWithAuthor[]>>(`${slug}/authors/${authorId}/emails`, {
    searchParams: params ? { params } : undefined,
  })
}

export async function getAuthorThreadsStarted(
  slug: string,
  authorId: number,
  params?: PaginationParams
): Promise<PaginatedResponse<ThreadWithStarter[]>> {
  return getJson<PaginatedResponse<ThreadWithStarter[]>>(
    `${slug}/authors/${authorId}/threads-started`,
    {
      searchParams: params ? { params } : undefined,
    }
  )
}

export async function getAuthorThreadsParticipated(
  slug: string,
  authorId: number,
  params?: PaginationParams
): Promise<PaginatedResponse<ThreadWithStarter[]>> {
  return getJson<PaginatedResponse<ThreadWithStarter[]>>(
    `${slug}/authors/${authorId}/threads-participated`,
    {
      searchParams: params ? { params } : undefined,
    }
  )
}

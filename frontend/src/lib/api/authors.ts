import { getJson } from "./http"
import { normalizePaginated, normalizeResponse } from "./adapters"
import type {
  ApiResponse,
  AuthorListParams,
  AuthorWithStats,
  EmailWithAuthor,
  NormalizedPaginatedResponse,
  NormalizedResponse,
  PaginationParams,
  ThreadWithStarter,
} from "./types"

export async function searchAuthors(
  slug: string,
  params?: AuthorListParams
): Promise<NormalizedPaginatedResponse<AuthorWithStats[]>> {
  const payload: AuthorListParams = {
    page: params?.page ?? 1,
    pageSize: params?.pageSize ?? 25,
    sort: params?.sort ?? [],
    q: params?.q ?? null,
    listSlug: params?.listSlug ?? slug,
  }

  const response = await getJson<ApiResponse<AuthorWithStats[]>>("authors", {
    searchParams: { params: payload },
  })

  return normalizePaginated(response)
}

export async function getAuthor(slug: string, authorId: number): Promise<NormalizedResponse<AuthorWithStats>> {
  const response = await getJson<ApiResponse<AuthorWithStats>>(`authors/${authorId}`)
  return normalizeResponse(response)
}

export async function getAuthorEmails(
  slug: string,
  authorId: number,
  params?: PaginationParams
): Promise<NormalizedPaginatedResponse<EmailWithAuthor[]>> {
  const pagination = withPaginationDefaults(params)
  const response = await getJson<ApiResponse<EmailWithAuthor[]>>(
    `authors/${authorId}/lists/${encodeURIComponent(slug)}/emails`,
    {
      searchParams: { params: pagination },
    }
  )
  return normalizePaginated(response)
}

export async function getAuthorThreadsStarted(
  slug: string,
  authorId: number,
  params?: PaginationParams
): Promise<NormalizedPaginatedResponse<ThreadWithStarter[]>> {
  const pagination = withPaginationDefaults(params)
  const response = await getJson<ApiResponse<ThreadWithStarter[]>>(
    `authors/${authorId}/lists/${encodeURIComponent(slug)}/threads-started`,
    {
      searchParams: { params: pagination },
    }
  )
  return normalizePaginated(response)
}

export async function getAuthorThreadsParticipated(
  slug: string,
  authorId: number,
  params?: PaginationParams
): Promise<NormalizedPaginatedResponse<ThreadWithStarter[]>> {
  const pagination = withPaginationDefaults(params)
  const response = await getJson<ApiResponse<ThreadWithStarter[]>>(
    `authors/${authorId}/lists/${encodeURIComponent(slug)}/threads-participated`,
    {
      searchParams: { params: pagination },
    }
  )
  return normalizePaginated(response)
}

function withPaginationDefaults(params?: PaginationParams): PaginationParams {
  return {
    page: params?.page ?? 1,
    pageSize: params?.pageSize ?? 25,
  }
}

import { getJson } from "./http"
import type {
  PaginatedResponse,
  ThreadDetail,
  ThreadListParams,
  ThreadSearchParams,
  ThreadSearchResponse,
  ThreadWithStarter,
} from "./types"

export async function listThreads(
  slug: string,
  params?: ThreadListParams
): Promise<PaginatedResponse<ThreadWithStarter[]>> {
  return getJson<PaginatedResponse<ThreadWithStarter[]>>(`${slug}/threads`, {
    searchParams: params ? { params } : undefined,
  })
}

export async function searchThreads(
  slug: string,
  params: ThreadSearchParams
): Promise<ThreadSearchResponse> {
  return getJson<ThreadSearchResponse>(`${slug}/threads/search`, {
    searchParams: { params },
  })
}

export async function getThread(slug: string, threadId: string): Promise<ThreadDetail> {
  return getJson<ThreadDetail>(`${slug}/threads/${threadId}`)
}

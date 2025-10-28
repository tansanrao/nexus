import { getJson } from "./http"
import { normalizePaginated, normalizeResponse } from "./adapters"
import type {
  ApiResponse,
  NormalizedPaginatedResponse,
  NormalizedResponse,
  ThreadDetail,
  ThreadListParams,
  ThreadWithStarter,
} from "./types"

export async function listThreads(
  slug: string,
  params?: ThreadListParams
): Promise<NormalizedPaginatedResponse<ThreadWithStarter[]>> {
  const response = await getJson<ApiResponse<ThreadWithStarter[]>>(`lists/${slug}/threads`, {
    searchParams: params ? { params } : undefined,
  })
  return normalizePaginated(response)
}

export async function getThread(
  slug: string,
  threadId: string
): Promise<NormalizedResponse<ThreadDetail>> {
  const response = await getJson<ApiResponse<ThreadDetail>>(`lists/${slug}/threads/${threadId}`)
  return normalizeResponse(response)
}

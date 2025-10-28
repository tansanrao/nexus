import { getJson, patchJson, postJson } from "./http"
import { normalizePaginated, normalizeResponse } from "./adapters"
import type {
  AdminMailingListRepository,
  AdminMailingListWithRepos,
  ApiResponse,
  ListQueryParams,
  MailingList,
  NormalizedPaginatedResponse,
  NormalizedResponse,
  ToggleRequest,
  ToggleResponse,
  SeedResponse,
} from "./types"

export async function listMailingLists(
  params?: ListQueryParams
): Promise<NormalizedPaginatedResponse<MailingList[]>> {
  const response = await getJson<ApiResponse<MailingList[]>>("lists", {
    searchParams: params ? { params } : undefined,
  })
  return normalizePaginated<MailingList[]>(response)
}

export async function getMailingList(slug: string): Promise<NormalizedResponse<MailingList>> {
  const response = await getJson<ApiResponse<MailingList>>(`lists/${slug}`)
  return normalizeResponse(response)
}

export async function getMailingListWithRepos(
  slug: string
): Promise<NormalizedResponse<AdminMailingListWithRepos>> {
  const response = await getJson<ApiResponse<AdminMailingListWithRepos>>(`lists/${slug}/repositories`, {
    client: "admin",
  })
  return normalizeResponse(response)
}

export async function toggleMailingList(slug: string, body: ToggleRequest): Promise<ToggleResponse> {
  const response = await patchJson<ApiResponse<ToggleResponse>>(`lists/${slug}/toggle`, body, {
    client: "admin",
  })
  return response.data
}

export async function seedMailingLists(): Promise<SeedResponse> {
  const response = await postJson<ApiResponse<SeedResponse>>("lists/seed", undefined, {
    client: "admin",
  })
  return response.data
}

export async function listMailingListRepositories(
  slug: string
): Promise<AdminMailingListRepository[]> {
  const { data } = await getMailingListWithRepos(slug)
  return data.repos ?? []
}

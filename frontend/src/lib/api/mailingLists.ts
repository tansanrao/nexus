import { getJson, patchJson, postJson } from "./http"
import type {
  DataResponse,
  MailingList,
  MailingListRepository,
  MailingListWithRepos,
  ToggleRequest,
  ToggleResponse,
  SeedResponse,
} from "./types"

export async function listMailingLists(): Promise<MailingList[]> {
  const response = await getJson<DataResponse<MailingList[]>>("admin/mailing-lists")
  return response.data
}

export async function getMailingList(slug: string): Promise<MailingList> {
  return getJson<MailingList>(`admin/mailing-lists/${slug}`)
}

export async function getMailingListWithRepos(slug: string): Promise<MailingListWithRepos> {
  return getJson<MailingListWithRepos>(`admin/mailing-lists/${slug}/repositories`)
}

export async function toggleMailingList(slug: string, body: ToggleRequest): Promise<ToggleResponse> {
  return patchJson<ToggleResponse>(`admin/mailing-lists/${slug}/toggle`, body)
}

export async function seedMailingLists(): Promise<SeedResponse> {
  return postJson<SeedResponse>("admin/mailing-lists/seed")
}

export async function listMailingListRepositories(slug: string): Promise<MailingListRepository[]> {
  const list = await getMailingListWithRepos(slug)
  return list.repos ?? []
}

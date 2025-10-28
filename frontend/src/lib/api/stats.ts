import { getJson } from "./http"
import { normalizeResponse } from "./adapters"
import type {
  ApiResponse,
  ListAggregateStats,
  MailingListStats,
  NormalizedResponse,
} from "./types"

export async function getMailingListStats(
  slug: string
): Promise<NormalizedResponse<MailingListStats>> {
  const response = await getJson<ApiResponse<MailingListStats>>(`lists/${slug}/stats`)
  return normalizeResponse(response)
}

export async function getAggregateStats(): Promise<NormalizedResponse<ListAggregateStats>> {
  const response = await getJson<ApiResponse<ListAggregateStats>>("lists/stats")
  return normalizeResponse(response)
}

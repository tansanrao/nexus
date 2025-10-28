import { getJson } from "./http"
import { normalizeResponse } from "./adapters"
import type { ApiResponse, EmailWithAuthor, NormalizedResponse } from "./types"

export async function getEmail(
  slug: string,
  emailId: number
): Promise<NormalizedResponse<EmailWithAuthor>> {
  const response = await getJson<ApiResponse<EmailWithAuthor>>(
    `lists/${encodeURIComponent(slug)}/emails/${emailId}`
  )
  return normalizeResponse(response)
}

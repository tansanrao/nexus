import { getJson } from "./http"
import type { EmailHierarchy } from "./types"

export async function getEmail(slug: string, emailId: number): Promise<EmailHierarchy> {
  return getJson<EmailHierarchy>(`${slug}/emails/${emailId}`)
}

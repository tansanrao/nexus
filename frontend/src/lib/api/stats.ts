import { getJson } from "./http"
import type { StatsOverview } from "./types"

export async function getStats(slug: string): Promise<StatsOverview> {
  return getJson<StatsOverview>(`${slug}/stats`)
}

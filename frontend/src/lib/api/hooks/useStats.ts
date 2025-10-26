import { useQuery } from "@tanstack/react-query"
import { getStats } from "../stats"
import { queryKeys } from "../queryKeys"

export function useStats(slug: string | undefined) {
  return useQuery({
    queryKey: slug ? queryKeys.stats.summary(slug) : ["stats", "summary", "empty"],
    queryFn: () => {
      if (!slug) {
        throw new Error("slug is required")
      }
      return getStats(slug)
    },
    enabled: Boolean(slug),
    staleTime: 1000 * 60 * 5,
  })
}

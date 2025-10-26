import { useQuery } from "@tanstack/react-query"
import { getThread, listThreads, searchThreads } from "../threads"
import type { ThreadListParams, ThreadSearchParams } from "../types"
import { queryKeys } from "../queryKeys"

export function useThreadsList(slug: string | undefined, params?: ThreadListParams) {
  return useQuery({
    queryKey: slug ? queryKeys.threads.list(slug, params) : ["threads", "list", "empty"],
    queryFn: () => {
      if (!slug) {
        throw new Error("slug is required")
      }
      return listThreads(slug, params)
    },
    enabled: Boolean(slug),
    staleTime: 1000 * 60, // 1 minute
  })
}

export function useThreadDetail(slug: string | undefined, threadId: string | undefined) {
  return useQuery({
    queryKey: slug && threadId ? queryKeys.threads.detail(slug, threadId) : ["threads", "detail", "empty"],
    queryFn: () => {
      if (!slug || !threadId) {
        throw new Error("slug and threadId are required")
      }
      return getThread(slug, threadId)
    },
    enabled: Boolean(slug && threadId),
    staleTime: 1000 * 30,
  })
}

export function useThreadSearch(slug: string | undefined, params: ThreadSearchParams | undefined) {
  return useQuery({
    queryKey: slug && params ? queryKeys.threads.search(slug, params) : ["threads", "search", "empty"],
    queryFn: () => {
      if (!slug || !params) {
        throw new Error("slug and params are required")
      }
      return searchThreads(slug, params)
    },
    enabled: Boolean(slug && params),
    staleTime: 1000 * 30,
  })
}

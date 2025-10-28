import { useMemo } from "react"

import { useQuery } from "@tanstack/react-query"

import { DEV_MODE_MAX_EMAILS_PER_THREAD, DEV_MODE_MAX_THREAD_PAGES } from "@src/lib/devMode"
import { useDevMode } from "@src/providers/DevModeProvider"

import { getThread, listThreads } from "../threads"
import type {
  NormalizedPaginatedResponse,
  NormalizedResponse,
  ThreadDetail,
  ThreadListParams,
  ThreadWithStarter,
} from "../types"
import { queryKeys } from "../queryKeys"

export function useThreadsList(slug: string | undefined, params?: ThreadListParams) {
  const { isDevMode } = useDevMode()

  const selectThreadList = useMemo(
    () =>
      (response: NormalizedPaginatedResponse<ThreadWithStarter[]>) =>
        isDevMode ? limitThreadsResponse(response) : response,
    [isDevMode]
  )

  return useQuery({
    queryKey: slug ? queryKeys.threads.list(slug, params) : ["threads", "list", "empty"],
    queryFn: () => {
      if (!slug) {
        throw new Error("slug is required")
      }
      return listThreads(slug, params)
    },
    enabled: Boolean(slug),
    staleTime: 1000 * 60,
    select: selectThreadList,
  })
}

export function useThreadDetail(slug: string | undefined, threadId: string | undefined) {
  const { isDevMode } = useDevMode()

  const selectThreadDetail = useMemo(
    () =>
      (response: NormalizedResponse<ThreadDetail>) =>
        isDevMode ? limitThreadDetail(response.data) : response.data,
    [isDevMode]
  )

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
    select: selectThreadDetail,
  })
}

export function useThreadSearch(
  slug: string | undefined,
  params: Record<string, unknown> | undefined
) {
  const key = slug && params
    ? ["threads", slug, "search", JSON.stringify(params)]
    : ["threads", "search", "unsupported"]

  return useQuery({
    queryKey: key,
    queryFn: async () => {
      throw new Error("Thread search is not available on the current API.")
    },
    enabled: false,
  })
}

function limitThreadsResponse(
  response: NormalizedPaginatedResponse<ThreadWithStarter[]>
): NormalizedPaginatedResponse<ThreadWithStarter[]> {
  const { pagination, data } = response
  const maxPages = Math.min(pagination.totalPages, DEV_MODE_MAX_THREAD_PAGES)
  const isPageWithinLimit = pagination.page <= DEV_MODE_MAX_THREAD_PAGES
  const limitedData = isPageWithinLimit ? data : []
  const maxItems = Math.min(pagination.totalItems, maxPages * pagination.pageSize)

  const updatedPagination = {
    ...pagination,
    page: isPageWithinLimit ? pagination.page : DEV_MODE_MAX_THREAD_PAGES,
    totalPages: maxPages,
    totalItems: maxItems,
  }

  return {
    ...response,
    data: limitedData,
    pagination: updatedPagination,
    meta: {
      ...response.meta,
      pagination: updatedPagination,
    },
  }
}

function limitThreadDetail(detail: ThreadDetail): ThreadDetail {
  if (!detail.emails) {
    return detail
  }

  return {
    ...detail,
    emails: detail.emails.slice(0, DEV_MODE_MAX_EMAILS_PER_THREAD),
  }
}

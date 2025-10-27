import { useMemo } from "react"

import { useQuery } from "@tanstack/react-query"

import { DEV_MODE_MAX_EMAILS_PER_THREAD, DEV_MODE_MAX_THREAD_PAGES } from "@src/lib/devMode"
import { useDevMode } from "@src/providers/DevModeProvider"

import { getThread, listThreads, searchThreads } from "../threads"
import type {
  PaginatedResponse,
  ThreadDetail,
  ThreadListParams,
  ThreadSearchParams,
  ThreadWithStarter,
} from "../types"
import { queryKeys } from "../queryKeys"

export function useThreadsList(slug: string | undefined, params?: ThreadListParams) {
  const { isDevMode } = useDevMode()

  const selectThreadList = useMemo(
    () =>
      (response: PaginatedResponse<ThreadWithStarter[]>) =>
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
    staleTime: 1000 * 60, // 1 minute
    select: selectThreadList,
  })
}

export function useThreadDetail(slug: string | undefined, threadId: string | undefined) {
  const { isDevMode } = useDevMode()

  const selectThreadDetail = useMemo(
    () => (response: ThreadDetail) => (isDevMode ? limitThreadDetail(response) : response),
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

function limitThreadsResponse(
  response: PaginatedResponse<ThreadWithStarter[]>
): PaginatedResponse<ThreadWithStarter[]> {
  const { page, data } = response
  const maxPages = Math.min(page.totalPages, DEV_MODE_MAX_THREAD_PAGES)
  const isPageWithinLimit = page.page <= DEV_MODE_MAX_THREAD_PAGES
  const limitedData = isPageWithinLimit ? data : []
  const maxElements = Math.min(page.totalElements, maxPages * page.size)

  return {
    data: limitedData,
    page: {
      ...page,
      page: isPageWithinLimit ? page.page : DEV_MODE_MAX_THREAD_PAGES,
      totalPages: maxPages,
      totalElements: maxElements,
    },
  }
}

function limitThreadDetail(response: ThreadDetail): ThreadDetail {
  if (!response.emails) {
    return response
  }

  return {
    ...response,
    emails: response.emails.slice(0, DEV_MODE_MAX_EMAILS_PER_THREAD),
  }
}

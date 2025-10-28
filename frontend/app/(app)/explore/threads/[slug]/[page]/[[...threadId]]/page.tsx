"use client"

import { useCallback, useEffect, useMemo, useState } from "react"
import {
  ReadonlyURLSearchParams,
  useParams,
  useRouter,
  useSearchParams,
} from "next/navigation"
import {
  IconAlertTriangle,
  IconGitBranch,
  IconLoader2,
  IconMessageCircle,
} from "@tabler/icons-react"

import {
  AppPageHeader,
  type AppBreadcrumbDropdownOption,
  type AppBreadcrumbItem,
} from "@/components/layouts/app-page-header"
import { ThreadBrowserLayout } from "@/components/thread-browser/thread-browser-layout"
import { ThreadDetailView } from "@/components/thread-browser/thread-detail-view"
import { ThreadDiffView } from "@/components/thread-browser/thread-diff-view"
import {
  ThreadListPanel,
  type ThreadListItem,
} from "@/components/thread-browser/thread-list-panel"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  useMailingLists,
  useThreadDetail,
  useThreadsList,
  useThreadSearch,
} from "@src/lib/api/hooks"
import {
  DEV_MODE_MAX_THREAD_PAGES,
  DEV_MODE_THREADS_PER_PAGE,
} from "@src/lib/devMode"
import { useDevMode } from "@src/providers/DevModeProvider"
import {
  isApiError,
  type NormalizedResponse,
  type PaginationMeta,
  type ThreadListParams,
  type ThreadSearchParams,
  type ThreadSearchPage,
} from "@src/lib/api"

const PAGE_SIZE = DEV_MODE_THREADS_PER_PAGE

type RouteParams = {
  slug: string
  page: string
  threadId?: string[] | string
}

export default function ThreadBrowserPage() {
  const params = useParams<RouteParams>()
  const searchParams = useSearchParams()
  const router = useRouter()

  const slug = decodeURIComponent(params.slug)
  const rawPageNumber = Number.parseInt(params.page, 10)
  const page = Number.isFinite(rawPageNumber) && rawPageNumber > 0 ? rawPageNumber : 1

  const rawThreadIdParam = params.threadId
  const threadIdParam = Array.isArray(rawThreadIdParam)
    ? rawThreadIdParam[0] ?? null
    : rawThreadIdParam ?? null

  const activeSearchQuery = useMemo(() => {
    const value = searchParams.get("q")
    return value ? value.trim() : ""
  }, [searchParams])

  const [searchInput, setSearchInput] = useState(activeSearchQuery)

    useEffect(() => {
    setSearchInput(activeSearchQuery)
  }, [activeSearchQuery])

  const isSearching = activeSearchQuery.length > 0

  useEffect(() => {
    if (!Number.isFinite(rawPageNumber) || rawPageNumber < 1) {
      const safePath = buildThreadPath(slug, page, threadIdParam, undefined, searchParams)
      router.replace(safePath)
    }
  }, [page, rawPageNumber, router, searchParams, slug, threadIdParam])

  const [activeView, setActiveView] = useState<"thread" | "diff">("thread")

  useEffect(() => {
    if (!threadIdParam) {
      setActiveView("thread")
    }
  }, [threadIdParam])

  const { isDevMode } = useDevMode()

  useEffect(() => {
    if (!isDevMode) {
      return
    }

    if (page > DEV_MODE_MAX_THREAD_PAGES) {
      router.replace(
        buildThreadPath(slug, DEV_MODE_MAX_THREAD_PAGES, null, undefined, searchParams)
      )
    }
  }, [isDevMode, page, router, searchParams, slug])

  const {
    data: mailingLists,
    isLoading: mailingListsLoading,
    isError: mailingListsError,
    error: mailingListsErrorValue,
    refetch: refetchMailingLists,
  } = useMailingLists()

  const listParams = useMemo<ThreadListParams>(
    () => ({
      page,
      pageSize: PAGE_SIZE,
      sort: ["last_date:desc"],
    }),
    [page]
  )

  const {
    data: threadResponse,
    isLoading: threadsLoading,
    isFetching: threadsFetching,
    isError: threadsError,
    error: threadsErrorValue,
    refetch: refetchThreads,
  } = useThreadsList(slug, listParams)

  const searchRequestParams = useMemo<ThreadSearchParams | undefined>(
    () => {
      if (!isSearching) {
        return undefined
      }
      return {
        q: activeSearchQuery,
        page,
        size: PAGE_SIZE,
        startDate: null,
        endDate: null,
        semanticRatio: null,
        hasPatches: null,
        starterId: null,
        participantId: [],
        seriesId: null,
        sort: ["last_activity:desc"],
        mailingList: [],
      }
    },
    [activeSearchQuery, isSearching, page]
  )

  const {
    data: threadSearchResponse,
    isLoading: searchLoading,
    isFetching: searchFetching,
    isError: searchError,
    error: searchErrorValue,
    refetch: refetchSearch,
  } = useThreadSearch(slug, searchRequestParams)

  const normalizedSearchResponse = threadSearchResponse as
    | NormalizedResponse<ThreadSearchPage>
    | undefined

  const {
    data: threadDetail,
    isLoading: threadDetailLoading,
    isError: threadDetailError,
    error: threadDetailErrorValue,
  } = useThreadDetail(slug, threadIdParam ?? undefined)

  const listThreadsData = threadResponse?.data ?? []

  const threadItems = useMemo<ThreadListItem[]>(() => {
    if (isSearching) {
      const responseData = normalizedSearchResponse?.data
      if (!responseData) {
        return []
      }

      return responseData.hits.map((hit) => {
        const summary = hit.thread
        const highlight =
          hit.highlights?.discussionText ??
          hit.highlights?.subjectText ??
          hit.firstPostExcerpt ??
          null

        return {
          id: summary.threadId,
          subject: summary.subject,
          messageCount: summary.messageCount,
          startDate: summary.startDate,
          lastActivity: summary.lastActivity,
          starterName: summary.starterName ?? null,
          starterEmail: summary.starterEmail,
          highlight,
          score: hit.score?.rankingScore ?? null,
          isSearchResult: true,
        }
      })
    }

    const listData = threadResponse?.data ?? []
    return listData.map((thread) => ({
      id: thread.id,
      subject: thread.subject,
      messageCount: thread.message_count ?? 0,
      startDate: thread.start_date,
      lastActivity: thread.last_date ?? thread.start_date,
      starterName: thread.starter_name ?? null,
      starterEmail: thread.starter_email,
    }))
  }, [isSearching, threadSearchResponse, threadResponse])

  const listPagination = threadResponse?.pagination

  const searchPagination: PaginationMeta | undefined = useMemo(() => {
    if (!isSearching) {
      return undefined
    }

    if (normalizedSearchResponse?.meta.pagination) {
      return normalizedSearchResponse.meta.pagination
    }

    const total = normalizedSearchResponse?.data.total ?? 0
    const size = searchRequestParams?.size ?? PAGE_SIZE
    const totalPages = Math.max(1, Math.ceil(total / size))
    const currentPage = searchRequestParams?.page ?? 1

    return {
      page: currentPage,
      pageSize: size,
      totalPages,
      totalItems: total,
    }
  }, [isSearching, searchRequestParams, threadSearchResponse])

  let effectivePagination: PaginationMeta = isSearching
    ? searchPagination ?? {
        page,
        pageSize: PAGE_SIZE,
        totalPages: 1,
        totalItems: normalizedSearchResponse?.data.total ?? 0,
      }
    : listPagination ?? {
        page,
        pageSize: PAGE_SIZE,
        totalPages: 1,
        totalItems: listThreadsData.length,
      }

  if (isDevMode) {
    const limitedTotalPages = Math.min(effectivePagination.totalPages, DEV_MODE_MAX_THREAD_PAGES)
    const limitedTotalItems = Math.min(
      effectivePagination.totalItems,
      limitedTotalPages * effectivePagination.pageSize
    )

    effectivePagination = {
      ...effectivePagination,
      page: Math.min(effectivePagination.page, DEV_MODE_MAX_THREAD_PAGES),
      totalPages: limitedTotalPages,
      totalItems: limitedTotalItems,
    }
  }

  const totalPages = Math.max(1, effectivePagination.totalPages)
  const totalItems = effectivePagination.totalItems
  const currentPageFromApi = Math.min(effectivePagination.page, totalPages)

  const selectedThreadIdNumber =
    threadIdParam && !Number.isNaN(Number(threadIdParam)) ? Number(threadIdParam) : null

  const selectedMailingList = useMemo(
    () => mailingLists?.find((list) => list.slug === slug),
    [mailingLists, slug]
  )
  const enabledMailingLists = useMemo(
    () => mailingLists?.filter((list) => list.enabled) ?? [],
    [mailingLists]
  )
  const hasUnknownSlug =
    mailingLists && !enabledMailingLists.some((list) => list.slug === slug)

  const handleSlugChange = useCallback(
    (nextSlug: string) => {
      router.push(buildThreadPath(nextSlug, 1, null, { search: activeSearchQuery || null }))
    },
    [activeSearchQuery, router]
  )

  const breadcrumbItems = useMemo<AppBreadcrumbItem[]>(() => {
    const dropdownItems: AppBreadcrumbDropdownOption[] = enabledMailingLists.map((list) => ({
      id: list.slug,
      label: list.slug,
      isActive: list.slug === slug,
      onSelect: () => handleSlugChange(list.slug),
    }))

    if (hasUnknownSlug && !dropdownItems.some((item) => item.id === slug)) {
      dropdownItems.unshift({
        id: slug,
        label: slug,
        disabled: true,
        isActive: true,
      })
    }

    return [
      {
        type: "link",
        label: "Explore",
        href: "/explore",
        hideOnMobile: true,
      },
      {
        type: "link",
        label: "Threads",
        href: "/explore/threads",
        hideOnMobile: true,
      },
      {
        type: "dropdown",
        label:
          mailingListsLoading && !mailingLists
            ? "Loading…"
            : selectedMailingList?.slug ?? slug,
        display: "label",
        items: dropdownItems.length
          ? dropdownItems
          : [
              {
                id: "loading",
                label: "Loading mailing lists…",
                disabled: true,
              },
            ],
        disabled: mailingListsLoading && !mailingLists,
        align: "end",
      },
    ]
  }, [
    enabledMailingLists,
    handleSlugChange,
    hasUnknownSlug,
    mailingLists,
    mailingListsLoading,
    selectedMailingList,
    slug,
  ])

  const handlePageChange = useCallback(
    (nextPage: number) => {
      router.push(buildThreadPath(slug, nextPage, null, undefined, searchParams))
    },
    [router, searchParams, slug]
  )

  const handleThreadSelect = useCallback(
    (threadId: number) => {
      router.push(buildThreadPath(slug, page, String(threadId), undefined, searchParams))
    },
    [page, router, searchParams, slug]
  )

  const handleClearSelection = useCallback(() => {
    router.push(buildThreadPath(slug, page, null, undefined, searchParams))
  }, [page, router, searchParams, slug])

  const handleSearchInputChange = useCallback((value: string) => {
    setSearchInput(value)
  }, [])

  const handleSearchSubmit = useCallback(() => {
    const trimmed = searchInput.trim()
    router.push(buildThreadPath(slug, 1, null, { search: trimmed || null }, searchParams))
  }, [router, searchInput, searchParams, slug])

  const handleSearchClear = useCallback(() => {
    setSearchInput("")
    if (activeSearchQuery) {
      router.push(buildThreadPath(slug, 1, null, { search: null }, searchParams))
    }
  }, [activeSearchQuery, router, searchParams, slug])

  const toolbar = (
    <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
      <div className="flex flex-wrap items-center gap-2 text-sm">
        <span className="font-medium">
          {selectedMailingList?.name ?? selectedMailingList?.slug ?? slug}
        </span>
        <Badge variant="outline" className="uppercase tracking-wide">
          {PAGE_SIZE} per page
        </Badge>
        {(isSearching ? searchFetching : threadsFetching) ? (
          <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
            <IconLoader2 className="size-3 animate-spin" />
            Refreshing
          </span>
        ) : null}
      </div>
      <div className="flex items-center gap-2">
        <Button
          variant={activeView === "thread" ? "default" : "outline"}
          size="sm"
          onClick={() => setActiveView("thread")}
        >
          <IconMessageCircle className="mr-1 size-4" />
          Conversation
        </Button>
        <Button
          variant={activeView === "diff" ? "default" : "outline"}
          size="sm"
          onClick={() => setActiveView("diff")}
          disabled={!threadIdParam}
        >
          <IconGitBranch className="mr-1 size-4" />
          Combined diff
        </Button>
      </div>
    </div>
  )

  const threadsLoadingState = isSearching
    ? searchLoading && !threadSearchResponse
    : threadsLoading && !threadResponse
  const threadsFetchingState = isSearching ? searchFetching : threadsFetching
  const threadsErrorState = isSearching ? searchError : threadsError
  const threadsErrorValueState = isSearching ? searchErrorValue : threadsErrorValue
  const refetchActive = isSearching ? refetchSearch : refetchThreads
  const errorTitle = isSearching ? "Unable to search threads" : "Unable to load threads"

  return (
    <div className="flex h-full w-full flex-col overflow-hidden">
      <AppPageHeader items={breadcrumbItems} />
      <div className="flex flex-1 flex-col gap-4 overflow-hidden px-4 py-4">
        {mailingListsError ? (
          <Alert variant="destructive">
            <IconAlertTriangle className="size-4" />
            <AlertTitle>Unable to load mailing lists</AlertTitle>
            <AlertDescription className="flex flex-col gap-2">
              <span>{formatError(mailingListsErrorValue)}</span>
              <div>
                <Button size="sm" variant="outline" onClick={() => refetchMailingLists()}>
                  Retry
                </Button>
              </div>
            </AlertDescription>
          </Alert>
        ) : null}

        {hasUnknownSlug ? (
          <Alert variant="destructive">
            <IconAlertTriangle className="size-4" />
            <AlertTitle>Unknown mailing list</AlertTitle>
            <AlertDescription>
              We couldn’t find {" "}
              <span className="font-medium text-foreground">{slug}</span> in the available lists.
              Pick another mailing list from the dropdown to continue.
            </AlertDescription>
          </Alert>
        ) : null}

        {threadsErrorState ? (
          <Alert variant="destructive">
            <IconAlertTriangle className="size-4" />
            <AlertTitle>{errorTitle}</AlertTitle>
            <AlertDescription className="flex flex-col gap-2">
              <span>{formatError(threadsErrorValueState)}</span>
              <div>
                <Button size="sm" variant="outline" onClick={() => refetchActive()}>
                  Retry
                </Button>
              </div>
            </AlertDescription>
          </Alert>
        ) : null}

        <div className="flex min-h-0 flex-1 overflow-hidden rounded-xl border border-border bg-background shadow-sm">
          <ThreadBrowserLayout
            toolbar={toolbar}
            sidebar={
              <ThreadListPanel
                items={threadItems}
                isLoading={threadsLoadingState}
                isFetching={threadsFetchingState}
                selectedThreadId={selectedThreadIdNumber}
                onSelect={(item) => handleThreadSelect(item.id)}
                page={currentPageFromApi}
                totalPages={totalPages}
                totalItems={totalItems}
                onPageChange={handlePageChange}
                searchValue={searchInput}
                onSearchChange={handleSearchInputChange}
                onSearchSubmit={handleSearchSubmit}
                onSearchClear={handleSearchClear}
                isSearchActive={isSearching}
                isSearchPending={isSearching && searchLoading}
                mode={isSearching ? "search" : "list"}
              />
            }
            content={
              activeView === "diff" ? (
                <ThreadDiffView
                  key={threadIdParam ?? "thread-diff"}
                  selectedThreadId={threadIdParam}
                  threadDetail={threadDetail}
                  isLoading={threadDetailLoading}
                  error={threadDetailError ? threadDetailErrorValue : null}
                  onShowConversation={() => setActiveView("thread")}
                />
              ) : (
                <ThreadDetailView
                  key={threadIdParam ?? "thread-detail"}
                  selectedThreadId={threadIdParam}
                  threadDetail={threadDetail}
                  isLoading={threadDetailLoading}
                  error={threadDetailError ? threadDetailErrorValue : null}
                  onClearSelection={handleClearSelection}
                />
              )
            }
          />
        </div>
      </div>
    </div>
  )

  function formatError(error: unknown) {
    if (isApiError(error)) {
      return `${error.message} (${error.status || "network"})`
    }
    if (error instanceof Error) {
      return error.message
    }
    return "Something went wrong. Please try again."
  }
}

function buildThreadPath(
  slug: string,
  page: number,
  threadId: string | null,
  options?: { search?: string | null },
  currentParams?: ReadonlyURLSearchParams
) {
  const encodedSlug = encodeURIComponent(slug)
  const segments = [`/explore/threads/${encodedSlug}`, page.toString()]
  if (threadId) {
    segments.push(encodeURIComponent(threadId))
  }

  const query = new URLSearchParams(currentParams ? currentParams.toString() : "")

  if (options && Object.prototype.hasOwnProperty.call(options, "search")) {
    const trimmed = options.search?.trim() ?? ""
    if (trimmed) {
      query.set("q", trimmed)
    } else {
      query.delete("q")
    }
  }

  const path = segments.join("/")
  const queryString = query.toString()
  return queryString ? `${path}?${queryString}` : path
}

"use client"

import { useCallback, useEffect, useMemo, useState } from "react"
import { useParams, useRouter } from "next/navigation"
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
import { ThreadListPanel } from "@/components/thread-browser/thread-list-panel"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  useMailingLists,
  useThreadDetail,
  useThreadsList,
} from "@src/lib/api/hooks"
import { DEV_MODE_MAX_THREAD_PAGES, DEV_MODE_THREADS_PER_PAGE } from "@src/lib/devMode"
import { useDevMode } from "@src/providers/DevModeProvider"
import { isApiError, type ThreadListParams } from "@src/lib/api"

const PAGE_SIZE = DEV_MODE_THREADS_PER_PAGE

type RouteParams = {
  slug: string
  page: string
  threadId?: string[] | string
}

export default function ThreadBrowserPage() {
  const params = useParams<RouteParams>()
  const router = useRouter()

  const slug = decodeURIComponent(params.slug)
  const rawPageNumber = Number.parseInt(params.page, 10)
  const page = Number.isFinite(rawPageNumber) && rawPageNumber > 0 ? rawPageNumber : 1

  const rawThreadIdParam = params.threadId
  const threadIdParam = Array.isArray(rawThreadIdParam)
    ? rawThreadIdParam[0] ?? null
    : rawThreadIdParam ?? null

  useEffect(() => {
    if (!Number.isFinite(rawPageNumber) || rawPageNumber < 1) {
      const safePath = buildThreadPath(slug, page, threadIdParam)
      router.replace(safePath)
    }
  }, [page, rawPageNumber, router, slug, threadIdParam])

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
      router.replace(buildThreadPath(slug, DEV_MODE_MAX_THREAD_PAGES, null))
    }
  }, [isDevMode, page, router, slug])

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
      size: PAGE_SIZE,
      sortBy: "lastDate",
      order: "desc",
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

  const {
    data: threadDetail,
    isLoading: threadDetailLoading,
    isError: threadDetailError,
    error: threadDetailErrorValue,
  } = useThreadDetail(slug, threadIdParam ?? undefined)

  const threads = threadResponse?.data ?? []
  const totalPages = threadResponse?.page.totalPages ?? 1
  const totalElements = threadResponse?.page.totalElements ?? 0
  const selectedThreadIdNumber =
    threadIdParam && !Number.isNaN(Number(threadIdParam))
      ? Number(threadIdParam)
      : null

  const selectedMailingList = mailingLists?.find((list) => list.slug === slug)
  const enabledMailingLists = mailingLists?.filter((list) => list.enabled) ?? []
  const hasUnknownSlug =
    mailingLists && !enabledMailingLists.some((list) => list.slug === slug)

  const handleSlugChange = useCallback(
    (nextSlug: string) => {
      router.push(buildThreadPath(nextSlug, 1, null))
    },
    [router]
  )

  const breadcrumbItems = useMemo<AppBreadcrumbItem[]>(() => {
    const dropdownItems: AppBreadcrumbDropdownOption[] = enabledMailingLists.map(
      (list) => ({
        id: list.slug,
        label: list.slug,
        isActive: list.slug === slug,
        onSelect: () => handleSlugChange(list.slug),
      })
    )

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
        href: "/app/explore",
        hideOnMobile: true,
      },
      {
        type: "link",
        label: "Threads",
        href: "/app/explore/threads",
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
    slug,
  ])

  const handlePageChange = useCallback(
    (nextPage: number) => {
      router.push(buildThreadPath(slug, nextPage, null))
    },
    [router, slug]
  )

  const handleThreadSelect = useCallback(
    (threadId: number) => {
      router.push(buildThreadPath(slug, page, String(threadId)))
    },
    [page, router, slug]
  )

  const handleClearSelection = useCallback(() => {
    router.push(buildThreadPath(slug, page, null))
  }, [page, router, slug])

  const toolbar = (
    <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
      <div className="flex flex-wrap items-center gap-2 text-sm">
        <span className="font-medium">
          {selectedMailingList?.name ?? selectedMailingList?.slug ?? slug}
        </span>
        <Badge variant="outline" className="uppercase tracking-wide">
          {PAGE_SIZE} per page
        </Badge>
        {threadsFetching ? (
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
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => refetchMailingLists()}
                >
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
              We couldn’t find{" "}
              <span className="font-medium text-foreground">{slug}</span> in the
              available lists. Pick another mailing list from the dropdown to
              continue.
            </AlertDescription>
          </Alert>
        ) : null}

        {threadsError ? (
          <Alert variant="destructive">
            <IconAlertTriangle className="size-4" />
            <AlertTitle>Unable to load threads</AlertTitle>
            <AlertDescription className="flex flex-col gap-2">
              <span>{formatError(threadsErrorValue)}</span>
              <div>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => refetchThreads()}
                >
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
                threads={threads}
                isLoading={threadsLoading && !threadResponse}
                isFetching={threadsFetching}
                selectedThreadId={selectedThreadIdNumber}
                onSelect={(thread) => handleThreadSelect(thread.id)}
                page={page}
                totalPages={totalPages}
                totalElements={totalElements}
                onPageChange={handlePageChange}
              />
            }
            content={
              activeView === "diff" ? (
                <ThreadDiffView
                  selectedThreadId={threadIdParam}
                  threadDetail={threadDetail}
                  isLoading={threadDetailLoading}
                  error={threadDetailError ? threadDetailErrorValue : null}
                  onShowConversation={() => setActiveView("thread")}
                />
              ) : (
                <ThreadDetailView
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

function buildThreadPath(slug: string, page: number, threadId: string | null) {
  const encodedSlug = encodeURIComponent(slug)
  const segments = [`/app/explore/threads/${encodedSlug}`, page.toString()]
  if (threadId) {
    segments.push(encodeURIComponent(threadId))
  }
  return segments.join("/")
}

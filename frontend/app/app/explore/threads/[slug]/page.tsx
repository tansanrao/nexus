"use client"

import { useMemo, useState } from "react"
import { useParams, useRouter } from "next/navigation"
import { IconAlertTriangle, IconInfoCircle, IconLoader2 } from "@tabler/icons-react"

import { AppPageHeader } from "@/components/layouts/app-page-header"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Skeleton } from "@/components/ui/skeleton"
import { cn } from "@/lib/utils"
import { isApiError, type ThreadListParams } from "@src/lib/api"
import { useMailingLists, useThreadsList } from "@src/lib/api/hooks"

import { ThreadTable } from "./components/ThreadTable"

const DEFAULT_PAGE_SIZE = 25

export default function ThreadsBySlugPage() {
  const params = useParams<{ slug: string }>()
  const router = useRouter()

  const slug = decodeURIComponent(params.slug)
  const [page, setPage] = useState(1)

  const {
    data: mailingLists,
    isLoading: mailingListsLoading,
    isError: mailingListsError,
    error: mailingListsErrorValue,
    refetch: refetchMailingLists,
  } = useMailingLists()

  const listParams: ThreadListParams = useMemo(
    () => ({
      page,
      size: DEFAULT_PAGE_SIZE,
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

  const selectedMailingList = mailingLists?.find((list) => list.slug === slug)
  const hasUnknownSlug = mailingLists && !selectedMailingList

  const totalPages = threadResponse?.page.totalPages ?? 1
  const totalElements = threadResponse?.page.totalElements ?? 0

  const breadcrumbItems = [
    { label: "Explore", href: "/app/explore", hideOnMobile: true },
    { label: "Threads", href: "/app/explore/threads", hideOnMobile: true },
    {
      label: slug,
      render: (
        <Select
          value={selectedMailingList ? selectedMailingList.slug : slug}
          onValueChange={(value) => {
            setPage(1)
            router.push(`/app/explore/threads/${value}`)
          }}
          disabled={mailingListsLoading || Boolean(mailingListsError)}
        >
          <SelectTrigger
            className={cn(
              "h-8 w-[180px] text-sm",
              mailingListsLoading && "text-muted-foreground"
            )}
          >
            <SelectValue placeholder="Select mailing list" />
          </SelectTrigger>
          <SelectContent align="end">
            {mailingLists?.map((list) => (
              <SelectItem key={list.slug} value={list.slug}>
                {list.name ?? list.slug}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      ),
    },
  ]

  return (
    <div className="flex h-full flex-col">
      <AppPageHeader items={breadcrumbItems} />
      <main className="flex flex-1 flex-col overflow-hidden px-4 pb-4 pt-0">
        <div className="flex-1 pt-4">
          <div className="flex h-full flex-col gap-4 pb-4">
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
                  We couldn’t find <span className="font-medium">{slug}</span> in the available lists.
                  Pick another mailing list from the dropdown to continue.
                </AlertDescription>
              </Alert>
            ) : null}

            <Card className="flex flex-1 flex-col overflow-hidden">
              <CardHeader className="flex flex-row items-center justify-between gap-4">
                <div className="space-y-1">
                  <CardTitle className="text-lg font-semibold">
                    {selectedMailingList?.name ?? slug}
                  </CardTitle>
                  <p className="text-sm text-muted-foreground">
                    {selectedMailingList?.description ??
                      "Recent threads with the latest activity on this mailing list."}
                  </p>
                </div>
                {threadsFetching && (
                  <span className="text-xs text-muted-foreground flex items-center gap-1">
                    <IconLoader2 className="size-3 animate-spin" />
                    Refreshing…
                  </span>
                )}
              </CardHeader>
              <CardContent className="flex-1 overflow-hidden">
                {threadsError ? (
                  <Alert variant="destructive">
                    <IconAlertTriangle className="size-4" />
                    <AlertTitle>Unable to load threads</AlertTitle>
                    <AlertDescription className="flex flex-col gap-2">
                      <span>{formatError(threadsErrorValue)}</span>
                      <div>
                        <Button size="sm" variant="outline" onClick={() => refetchThreads()}>
                          Retry
                        </Button>
                      </div>
                    </AlertDescription>
                  </Alert>
                ) : mailingListsLoading && !mailingLists ? (
                  <div className="space-y-3">
                    <Skeleton className="h-6 w-1/3" />
                    <Skeleton className="h-32 w-full" />
                  </div>
                ) : (
                  <ThreadTable
                    threads={threadResponse?.data ?? []}
                    isLoading={threadsLoading && !threadResponse}
                  />
                )}
              </CardContent>
              <CardFooter className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                  <IconInfoCircle className="size-4 text-muted-foreground/80" />
                  <span>
                    Page {threadResponse?.page.page ?? page} of {totalPages} · {totalElements} threads total
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setPage((current) => Math.max(1, current - 1))}
                    disabled={page <= 1 || threadsLoading}
                  >
                    Previous
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setPage((current) => Math.min(totalPages, current + 1))}
                    disabled={page >= totalPages || threadsLoading}
                  >
                    Next
                  </Button>
                </div>
              </CardFooter>
            </Card>
          </div>
        </div>
      </main>
    </div>
  )
}

function formatError(error: unknown) {
  if (isApiError(error)) {
    return `${error.message} (${error.status || "network"})`
  }
  if (error instanceof Error) {
    return error.message
  }
  return "Something went wrong. Please try again."
}

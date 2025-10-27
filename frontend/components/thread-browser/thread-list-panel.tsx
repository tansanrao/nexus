"use client"

import { IconLoader2, IconSearch } from "@tabler/icons-react"
import { useMemo } from "react"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { ScrollArea } from "@/components/ui/scroll-area"
import { cn } from "@/lib/utils"
import { formatRelativeTime } from "@/lib/locale-format"
import type { ThreadWithStarter } from "@src/lib/api"

type ThreadListPanelProps = {
  threads: ThreadWithStarter[]
  isLoading: boolean
  isFetching?: boolean
  selectedThreadId: number | null
  onSelect: (thread: ThreadWithStarter) => void
  page: number
  totalPages: number
  totalElements: number
  onPageChange: (page: number) => void
}

export function ThreadListPanel({
  threads,
  isLoading,
  isFetching,
  selectedThreadId,
  onSelect,
  page,
  totalPages,
  totalElements,
  onPageChange,
}: ThreadListPanelProps) {
  const emptyMessage = useMemo(() => {
    if (isLoading) {
      return "Loading threads…"
    }
    if (threads.length === 0) {
      return "No threads found on this page."
    }
    return null
  }, [isLoading, threads.length])

  const safeTotalPages = Math.max(1, totalPages)
  const currentPage = Math.min(Math.max(1, page), safeTotalPages)

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex flex-col gap-2 border-b border-border bg-background px-3 py-3">
        <div className="flex items-center justify-between gap-2">
          <div>
            <p className="text-sm font-semibold">Threads</p>
            <p className="text-xs text-muted-foreground">
              {totalElements.toLocaleString()} total results
            </p>
          </div>
          {isFetching ? (
            <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
              <IconLoader2 className="size-3 animate-spin" />
              Refreshing
            </span>
          ) : null}
        </div>
        <div className="relative">
          <IconSearch className="absolute left-2 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="Search threads (coming soon)"
            className="pl-8"
            disabled
          />
          <Badge
            variant="outline"
            className="absolute right-2 top-1/2 -translate-y-1/2 text-[10px]"
          >
            Stub
          </Badge>
        </div>
      </div>
      <ScrollArea className="flex-1 min-h-0">
        <div className="flex flex-col">
          {isLoading ? (
            Array.from({ length: 10 }).map((_, index) => (
              <div
                key={`skeleton-${index}`}
                className="animate-pulse border-b border-border/40 px-3 py-3"
              >
                <div className="h-4 w-3/4 rounded bg-muted" />
                <div className="mt-2 h-3 w-1/2 rounded bg-muted" />
              </div>
            ))
          ) : emptyMessage ? (
            <div className="px-3 py-10 text-center text-sm text-muted-foreground">
              {emptyMessage}
            </div>
          ) : (
            threads.map((thread) => {
              const isSelected = selectedThreadId === thread.id
              return (
                <button
                  key={thread.id}
                  type="button"
                  onClick={() => onSelect(thread)}
                  className={cn(
                    "flex w-full flex-col items-start gap-2 border-b border-border/40 px-3 py-3 text-left transition",
                    "hover:bg-muted/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                    isSelected && "bg-muted/60"
                  )}
                >
                  <div className="flex w-full items-start justify-between gap-3">
                    <p className="line-clamp-2 text-sm font-medium text-foreground">
                      {thread.subject}
                    </p>
                    <span className="shrink-0 rounded border border-border/60 px-1.5 py-0.5 text-xs font-medium text-muted-foreground">
                      {thread.message_count ?? 0}
                    </span>
                  </div>
                  <div className="flex w-full flex-wrap items-center justify-between gap-2 text-xs text-muted-foreground">
                    <span className="truncate">
                      {thread.starter_name ?? thread.starter_email}
                    </span>
                    <span className="truncate text-right">
                      Updated{" "}
                      {formatRelativeTime(thread.last_date ?? thread.start_date)}
                    </span>
                  </div>
                </button>
              )
            })
          )}
        </div>
      </ScrollArea>
      <div className="mt-auto flex shrink-0 flex-col gap-2 border-t border-border bg-background px-3 py-3 text-xs text-muted-foreground">
        <div className="flex items-center justify-between">
          <span>
            Page {currentPage} of {safeTotalPages}
          </span>
          <span>{threads.length} rows</span>
        </div>
        <div className="flex items-center justify-between gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => onPageChange(Math.max(1, currentPage - 1))}
            disabled={currentPage <= 1 || isLoading}
          >
            Previous
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => onPageChange(Math.min(safeTotalPages, currentPage + 1))}
            disabled={currentPage >= safeTotalPages || isLoading}
          >
            Next
          </Button>
        </div>
      </div>
    </div>
  )
}

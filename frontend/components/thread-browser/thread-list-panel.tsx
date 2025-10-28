"use client"

import { IconLoader2, IconSearch, IconX } from "@tabler/icons-react"
import { useMemo } from "react"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { ScrollArea } from "@/components/ui/scroll-area"
import { cn } from "@/lib/utils"
import { formatRelativeTime } from "@/lib/locale-format"

export type ThreadListItem = {
  id: number
  subject: string
  messageCount: number
  startDate: string
  lastActivity: string
  starterName?: string | null
  starterEmail: string
  highlight?: string | null
  score?: number | null
  isSearchResult?: boolean
}

type ThreadListPanelProps = {
  items: ThreadListItem[]
  isLoading: boolean
  isFetching?: boolean
  selectedThreadId: number | null
  onSelect: (item: ThreadListItem) => void
  page: number
  totalPages: number
  totalItems: number
  onPageChange: (page: number) => void
  searchValue: string
  onSearchChange: (value: string) => void
  onSearchSubmit: () => void
  onSearchClear: () => void
  isSearchActive: boolean
  isSearchPending?: boolean
  mode: "list" | "search"
}

export function ThreadListPanel({
  items,
  isLoading,
  isFetching,
  selectedThreadId,
  onSelect,
  page,
  totalPages,
  totalItems,
  onPageChange,
  searchValue,
  onSearchChange,
  onSearchSubmit,
  onSearchClear,
  isSearchActive,
  isSearchPending,
  mode,
}: ThreadListPanelProps) {
  const emptyMessage = useMemo(() => {
    if (isLoading) {
      return mode === "search" ? "Searching threads…" : "Loading threads…"
    }
    if (items.length === 0) {
      return mode === "search" ? "No threads match your query." : "No threads found on this page."
    }
    return null
  }, [isLoading, items.length, mode])

  const safeTotalPages = Math.max(1, totalPages)
  const currentPage = Math.min(Math.max(1, page), safeTotalPages)
  const headerTitle = mode === "search" ? "Search results" : "Threads"

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex flex-col gap-2 border-b border-border bg-background px-3 py-3">
        <div className="flex items-center justify-between gap-2">
          <div>
            <p className="text-sm font-semibold">
              {headerTitle}
              {mode === "search" ? (
                <Badge className="ml-2 align-middle" variant="outline">
                  Search
                </Badge>
              ) : null}
            </p>
            <p className="text-xs text-muted-foreground">
              {totalItems.toLocaleString()} total results
            </p>
          </div>
          {isFetching ? (
            <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
              <IconLoader2 className="size-3 animate-spin" />
              Refreshing
            </span>
          ) : null}
        </div>
        <form
          className="relative flex items-center gap-2"
          onSubmit={(event) => {
            event.preventDefault()
            onSearchSubmit()
          }}
        >
          <div className="relative flex-1">
            <IconSearch className="absolute left-2 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="Search threads"
              className="pl-8"
              value={searchValue}
              onChange={(event) => onSearchChange(event.target.value)}
              disabled={isSearchPending}
            />
          </div>
          <Button type="submit" size="sm" disabled={isSearchPending}>
            Search
          </Button>
          {isSearchActive ? (
            <Button type="button" size="sm" variant="secondary" onClick={onSearchClear}>
              <IconX className="mr-1 size-4" />
              Clear
            </Button>
          ) : null}
        </form>
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
            items.map((item) => {
              const isSelected = selectedThreadId === item.id
              return (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => onSelect(item)}
                  className={cn(
                    "flex w-full flex-col items-start gap-2 border-b border-border/40 px-3 py-3 text-left transition",
                    "hover:bg-muted/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                    isSelected && "bg-muted/60"
                  )}
                >
                  <div className="flex w-full items-start justify-between gap-3">
                    <p className="line-clamp-2 text-sm font-medium text-foreground">
                      {item.subject}
                    </p>
                    <span className="shrink-0 rounded border border-border/60 px-1.5 py-0.5 text-xs font-medium text-muted-foreground">
                      {item.messageCount}
                    </span>
                  </div>
                  <div className="flex w-full flex-wrap items-center justify-between gap-2 text-xs text-muted-foreground">
                    <span className="truncate">
                      {item.starterName ?? item.starterEmail}
                    </span>
                    <span className="truncate text-right">
                      Updated {formatRelativeTime(item.lastActivity || item.startDate)}
                    </span>
                  </div>
                  {item.highlight ? (
                    <p className="line-clamp-2 text-xs text-muted-foreground/80">
                      {item.highlight}
                    </p>
                  ) : null}
                  {typeof item.score === "number" ? (
                    <span className="text-[10px] uppercase tracking-wide text-muted-foreground/70">
                      Score {item.score.toFixed(3)}
                    </span>
                  ) : null}
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
          <span>{items.length} rows</span>
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

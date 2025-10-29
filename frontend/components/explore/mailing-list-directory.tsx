"use client"

import { useMemo } from "react"
import Link from "next/link"
import { IconChevronRight } from "@tabler/icons-react"

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Skeleton } from "@/components/ui/skeleton"
import { cn } from "@/lib/utils"
import { isApiError } from "@src/lib/api/http"
import { useMailingLists } from "@src/lib/api/hooks/useMailingLists"
import type { MailingList } from "@src/lib/api"

export function MailingListDirectory() {
  const {
    data,
    isPending,
    isError,
    error,
  } = useMailingLists()

  const mailingLists = useMemo(() => {
    if (!data) {
      return [] as MailingList[]
    }
    return [...data]
      .filter((list) => list.enabled)
      .sort((a, b) => a.name.localeCompare(b.name))
  }, [data])

  if (isPending) {
    return (
      <div className="space-y-3">
        {Array.from({ length: 4 }).map((_, index) => (
          <div
            key={index}
            className="rounded-lg border border-border/80 bg-card px-4 py-3"
          >
            <Skeleton className="h-3 w-24" />
            <Skeleton className="mt-3 h-4 w-3/4" />
            <Skeleton className="mt-2 h-3 w-5/6" />
          </div>
        ))}
      </div>
    )
  }

  if (isError) {
    const message = isApiError(error)
      ? `${error.message} (${error.status || "network"})`
      : "We couldn’t load mailing lists right now. Please try again."

    return (
      <Alert variant="destructive" className="max-w-xl">
        <AlertTitle>Unable to load mailing lists</AlertTitle>
        <AlertDescription>{message}</AlertDescription>
      </Alert>
    )
  }

  if (mailingLists.length === 0) {
    return (
      <div className="rounded-xl border border-dashed border-border/70 bg-muted/10 px-6 py-10 text-center text-sm text-muted-foreground">
        No mailing lists are currently enabled for syncing. Check back after an
        administrator turns one on.
      </div>
    )
  }

  return (
    <ul className="space-y-3">
      {mailingLists.map((list) => (
        <li key={list.id}>
          <Link
            href={`/explore/threads/${list.slug}`}
            className={cn(
              "group flex items-center justify-between gap-4 rounded-lg border border-border/80 bg-card px-4 py-3 transition",
              "hover:border-primary/60 hover:bg-muted/20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            )}
          >
            <div className="min-w-0">
              <div className="flex items-center gap-2 text-xs font-medium uppercase tracking-wide text-muted-foreground">
                <span>Mailing list</span>
                <span className="text-muted-foreground/70">/{list.slug}</span>
              </div>
              <p className="mt-1 truncate text-sm font-semibold text-foreground">
                {list.name}
              </p>
              {list.description ? (
                <p className="mt-1 text-sm text-muted-foreground">
                  {list.description}
                </p>
              ) : null}
            </div>
            <IconChevronRight className="size-4 flex-shrink-0 text-muted-foreground transition group-hover:translate-x-1" />
          </Link>
        </li>
      ))}
    </ul>
  )
}

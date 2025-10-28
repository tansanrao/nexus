"use client"

import {
  IconArrowLeft,
  IconChevronDown,
  IconChevronRight,
  IconList,
  IconListTree,
} from "@tabler/icons-react"
import { useCallback, useMemo, useState } from "react"
import { usePathname, useRouter, useSearchParams } from "next/navigation"

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Separator } from "@/components/ui/separator"
import { cn } from "@/lib/utils"
import { extractDiffContent } from "@/lib/diff"
import { formatDate, formatDateTime, formatRelativeTime } from "@/lib/locale-format"
import type { ThreadDetail } from "@src/lib/api"

import { EmailBody } from "./email-body"
import { GitDiffViewer } from "./git-diff-viewer"

type ThreadDetailViewProps = {
  selectedThreadId: string | null
  threadDetail: ThreadDetail | undefined
  isLoading: boolean
  error: unknown
  onClearSelection: () => void
}

export function ThreadDetailView({
  selectedThreadId,
  threadDetail,
  isLoading,
  error,
  onClearSelection,
}: ThreadDetailViewProps) {
  const router = useRouter()
  const pathname = usePathname()
  const searchParams = useSearchParams()
  const [collapsedEmailIds, setCollapsedEmailIds] = useState<Set<number>>(
    () => new Set()
  )
  const [hideDeepCollapsedReplies, setHideDeepCollapsedReplies] =
    useState(true)

  const emails = useMemo(
    () => (threadDetail ? threadDetail.emails : []),
    [threadDetail]
  )
  const hiddenCounts = useMemo(() => computeHiddenCounts(emails), [emails])
  const emailsWithState = useMemo(() => {
    if (!hideDeepCollapsedReplies) {
      return emails.map((email) => ({
        email,
        isCollapsed: collapsedEmailIds.has(email.id),
        isHidden: false,
      }))
    }

    const result: Array<{
      email: ThreadDetail["emails"][number]
      isCollapsed: boolean
      isHidden: boolean
    }> = []
    const collapsedStack: number[] = []

    for (const email of emails) {
      while (
        collapsedStack.length > 0 &&
        email.depth <= collapsedStack[collapsedStack.length - 1]
      ) {
        collapsedStack.pop()
      }

      const isCollapsed = collapsedEmailIds.has(email.id)
      const hasCollapsedAncestor = collapsedStack.length > 0
      const isHidden = hasCollapsedAncestor && email.depth > 1

      result.push({ email, isCollapsed, isHidden })

      if (isCollapsed && email.depth >= 1) {
        collapsedStack.push(email.depth)
      }
    }

    return result
  }, [collapsedEmailIds, emails, hideDeepCollapsedReplies])

  const handleToggle = useCallback((emailId: number) => {
    setCollapsedEmailIds((prev) => {
      const next = new Set(prev)
      if (next.has(emailId)) {
        next.delete(emailId)
      } else {
        next.add(emailId)
      }
      return next
    })
  }, [])

  const collapseAll = useCallback(() => {
    setCollapsedEmailIds(new Set(emails.map((email) => email.id)))
  }, [emails])

  const expandAll = useCallback(() => {
    setCollapsedEmailIds(new Set())
  }, [])

  const toggleHideDeepCollapsedReplies = useCallback(() => {
    setHideDeepCollapsedReplies((prev) => !prev)
  }, [])

  const handleAuthorActivate = useCallback(
    (
      event: React.MouseEvent<HTMLSpanElement> | React.KeyboardEvent<HTMLSpanElement>,
      authorId: number | null
    ) => {
      if (!authorId) {
        return
      }

      if ("key" in event) {
        if (event.key !== "Enter" && event.key !== " ") {
          return
        }
        event.preventDefault()
      }

      event.stopPropagation()

      const nextParams = new URLSearchParams(searchParams.toString())
      nextParams.set("author", String(authorId))
      const nextUrl = `${pathname}?${nextParams.toString()}`
      router.push(nextUrl)
    },
    [pathname, router, searchParams]
  )

  if (!selectedThreadId) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-3 bg-muted/10 text-center">
        <div className="space-y-2 px-4">
          <p className="text-sm font-medium text-muted-foreground">
            Select a thread on the left to view the conversation.
          </p>
        </div>
      </div>
    )
  }

  if (isLoading) {
    return (
      <div className="flex h-full flex-col gap-6 bg-background px-6 py-6">
        <header className="space-y-2">
          <div className="h-6 w-3/4 animate-pulse rounded bg-muted" />
          <div className="h-4 w-1/3 animate-pulse rounded bg-muted/70" />
        </header>
        {Array.from({ length: 4 }).map((_, index) => (
          <div key={index} className="space-y-3">
            <div className="h-4 w-1/2 animate-pulse rounded bg-muted/70" />
            <div className="h-16 w-full animate-pulse rounded bg-muted/40" />
          </div>
        ))}
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex h-full flex-col items-center justify-center bg-background px-6 py-6">
        <Alert variant="destructive" className="max-w-md">
          <AlertTitle>Unable to load thread</AlertTitle>
          <AlertDescription>
            {error instanceof Error ? error.message : "Something went wrong."}
          </AlertDescription>
        </Alert>
        <Button variant="ghost" className="mt-4" onClick={onClearSelection}>
          Go back
        </Button>
      </div>
    )
  }

  if (!threadDetail) {
    return (
      <div className="flex h-full flex-col items-center justify-center bg-background px-6 py-6 text-sm text-muted-foreground">
        Thread details unavailable.
      </div>
    )
  }

  const { thread } = threadDetail

  return (
    <div className="flex h-full min-h-0 flex-col bg-background">
      <div className="flex shrink-0 flex-wrap items-start justify-between gap-3 border-b border-border px-6 py-5">
        <div className="space-y-2">
          <div className="flex items-center gap-3">
            <Button
              variant="ghost"
              size="sm"
              className="-ml-2"
              onClick={onClearSelection}
            >
              <IconArrowLeft className="size-4" />
            </Button>
            <h1 className="text-lg font-semibold leading-tight text-foreground">
              {thread.subject}
            </h1>
          </div>
          <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
            <Badge variant="outline" className="text-xs font-medium">
              {thread.message_count ?? 0}{" "}
              {(thread.message_count ?? 0) === 1 ? "message" : "messages"}
            </Badge>
            <Separator orientation="vertical" className="h-4" />
            <span>Started {formatDateTime(thread.start_date)}</span>
            {thread.last_date && thread.last_date !== thread.start_date ? (
              <>
                <Separator orientation="vertical" className="h-4" />
                <span>Last activity {formatDateTime(thread.last_date)}</span>
              </>
            ) : null}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={expandAll}>
            <IconChevronDown className="mr-1 size-4" />
            Expand all
          </Button>
          <Button variant="outline" size="sm" onClick={collapseAll}>
            <IconChevronRight className="mr-1 size-4 rotate-90" />
            Collapse all
          </Button>
          <Button
            variant={hideDeepCollapsedReplies ? "default" : "outline"}
            size="sm"
            onClick={toggleHideDeepCollapsedReplies}
            aria-pressed={hideDeepCollapsedReplies}
          >
            {hideDeepCollapsedReplies ? (
              <IconListTree className="mr-1 size-4" />
            ) : (
              <IconList className="mr-1 size-4" />
            )}
            {hideDeepCollapsedReplies ? "Hide deep replies" : "Show headers"}
          </Button>
        </div>
      </div>

      <div className="flex-1 overflow-auto">
        <div className="flex min-h-full flex-col gap-4 px-6 py-6">
          {emailsWithState.map(({ email, isCollapsed, isHidden }) => {
            if (isHidden) {
              return null
            }
            const diffContent = extractDiffContent(
              email.body,
              email.patch_metadata
            )
            const { cleanBody, parsedSubject } = parseEmailBody(
              email.body,
              email.patch_metadata
            )

            const indentation =
              email.depth > 0 ? Math.min(email.depth, 8) * 16 : 0

            const hiddenReplyCount = hiddenCounts.get(email.id) ?? 0

            return (
              <article
                key={email.id}
                className={cn(
                  "space-y-3 border-l-2 border-transparent pl-3",
                  isCollapsed && "opacity-90"
                )}
                style={{ marginLeft: indentation }}
              >
                <button
                  type="button"
                  onClick={() => handleToggle(email.id)}
                  className="flex w-full items-start gap-3 rounded-md px-3 py-2 text-left transition hover:bg-muted/30 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                  <span className="mt-1 text-muted-foreground">
                    {isCollapsed ? (
                      <IconChevronRight className="size-4" />
                    ) : (
                      <IconChevronDown className="size-4" />
                    )}
                  </span>
                  <div className="flex flex-1 flex-col gap-1">
                    <div className="flex flex-wrap items-center gap-2 text-sm text-foreground">
                      <span
                        role="button"
                        tabIndex={0}
                        onClick={(event) =>
                          handleAuthorActivate(event, email.author_id ?? null)
                        }
                        onKeyDown={(event) =>
                          handleAuthorActivate(event, email.author_id ?? null)
                        }
                        className="font-medium text-foreground underline-offset-2 hover:underline"
                      >
                        {email.author_name ?? email.author_email}
                      </span>
                      <Separator orientation="vertical" className="h-4" />
                      <span>{formatDate(email.date)}</span>
                      <Separator orientation="vertical" className="h-4" />
                      <span>{formatRelativeTime(email.date)}</span>
                    </div>
                    <p className="text-sm text-muted-foreground">
                      {parsedSubject ?? email.subject ?? "No subject"}
                    </p>
                    {isCollapsed && hiddenReplyCount > 0 ? (
                      <p className="text-xs text-muted-foreground">
                        [{hiddenReplyCount} more]
                      </p>
                    ) : null}
                  </div>
                </button>

                {!isCollapsed ? (
                  <div className="ml-7 space-y-3 border-l-2 border-muted pl-4">
                    {cleanBody ? <EmailBody body={cleanBody} /> : null}
                    {diffContent.trim().length > 0 ? (
                      <GitDiffViewer
                        diff={diffContent}
                        defaultExpanded={false}
                        gitCommitHash={email.git_commit_hash ?? null}
                      />
                    ) : null}
                    <p className="text-xs text-muted-foreground">
                      Message ID: {email.message_id}
                    </p>
                  </div>
                ) : null}
              </article>
            )
          })}
        </div>
      </div>
    </div>
  )
}

function computeHiddenCounts(emails: ThreadDetail["emails"]) {
  const counts = new Map<number, number>()
  for (let i = 0; i < emails.length; i += 1) {
    const current = emails[i]
    const currentDepth = current.depth
    let childCount = 0
    for (let j = i + 1; j < emails.length; j += 1) {
      const next = emails[j]
      if (next.depth <= currentDepth) {
        break
      }
      childCount += 1
    }
    counts.set(current.id, childCount)
  }
  return counts
}

function parseEmailBody(
  body: string | null | undefined,
  patchMetadata: ThreadDetail["emails"][number]["patch_metadata"]
) {
  if (!body) {
    return { cleanBody: "", parsedSubject: null as string | null }
  }

  const lines = body.split("\n")
  const cleanLines: string[] = []
  let parsedSubject: string | null = null

  const excludeLines = new Set<number>()
  if (patchMetadata && patchMetadata.diff_sections) {
    for (const section of patchMetadata.diff_sections) {
      for (let i = section.start_line; i <= section.end_line; i += 1) {
        excludeLines.add(i)
      }
    }
  }

  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i]
    if (line.trim().startsWith("From:")) {
      continue
    }
    if (line.trim().startsWith("Subject:")) {
      parsedSubject = line.replace(/^Subject:\s*/i, "").trim() || null
      continue
    }
    if (excludeLines.has(i)) {
      continue
    }
    cleanLines.push(line)
  }

  while (cleanLines.length > 0 && cleanLines[0].trim() === "") {
    cleanLines.shift()
  }

  return {
    cleanBody: cleanLines.join("\n"),
    parsedSubject,
  }
}

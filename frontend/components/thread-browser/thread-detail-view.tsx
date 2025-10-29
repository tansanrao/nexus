"use client"

import {
  IconArrowLeft,
  IconChevronDown,
  IconChevronRight,
  IconChevronUp,
  IconList,
  IconListTree,
} from "@tabler/icons-react"
import { useCallback, useMemo, useState } from "react"
import { usePathname, useRouter, useSearchParams } from "next/navigation"

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { cn } from "@/lib/utils"
import { extractDiffContent } from "@/lib/diff"
import { formatDate, formatRelativeTime } from "@/lib/locale-format"
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
    <div className="flex h-full min-h-0 flex-col bg-muted/5">
      <div className="flex shrink-0 flex-wrap items-start justify-between gap-3 border-b border-border/60 bg-background/90 px-6 py-4 backdrop-blur supports-[backdrop-filter]:bg-background/80">
        <div className="min-w-0 space-y-2">
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="icon"
              className="-ml-1 md:hidden"
              onClick={onClearSelection}
            >
              <IconArrowLeft className="size-4" />
              <span className="sr-only">Back to threads</span>
            </Button>
            <h1 className="text-lg font-semibold leading-tight text-foreground md:ml-0">
              {thread.subject}
            </h1>
          </div>
          <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
            <span>
              {thread.message_count ?? 0}{" "}
              {(thread.message_count ?? 0) === 1 ? "message" : "messages"}
            </span>
            <span>•</span>
            <span>Started {formatDate(thread.start_date)}</span>
            {thread.last_date && thread.last_date !== thread.start_date ? (
              <>
                <span>•</span>
                <span>Last activity {formatDate(thread.last_date)}</span>
              </>
            ) : null}
          </div>
        </div>
        <div className="flex items-center gap-1">
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={expandAll}
            title="Expand all messages"
            aria-label="Expand all messages"
          >
            <IconChevronDown className="size-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={collapseAll}
            title="Collapse all messages"
            aria-label="Collapse all messages"
          >
            <IconChevronUp className="size-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className={cn(
              "h-7 w-7",
              hideDeepCollapsedReplies && "bg-muted text-foreground"
            )}
            onClick={toggleHideDeepCollapsedReplies}
            aria-pressed={hideDeepCollapsedReplies}
            title={
              hideDeepCollapsedReplies
                ? "Collapsed replies hide deeper messages"
                : "Collapsed replies show message headers"
            }
          >
            {hideDeepCollapsedReplies ? (
              <IconListTree className="size-4" />
            ) : (
              <IconList className="size-4" />
            )}
          </Button>
        </div>
      </div>

      <ScrollArea className="flex-1">
        <div className="space-y-0 px-6 py-6">
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

            const indentPx =
              email.depth > 0 ? Math.min(email.depth, 8) * 16 : 0
            const indentationStyle = indentPx
              ? {
                  marginLeft: `${indentPx}px`,
                  maxWidth: `calc(100% - ${indentPx}px)`,
                }
              : undefined

            const hiddenReplyCount = hiddenCounts.get(email.id) ?? 0
            const displaySubject = parsedSubject ?? email.subject ?? "No subject"

            return (
              <div key={email.id} style={indentationStyle} className="min-w-0">
                <div className="px-3 py-2 rounded-md transition-colors hover:bg-muted/20">
                  <button
                    type="button"
                    onClick={() => handleToggle(email.id)}
                    className="flex w-full items-start gap-2 text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/60"
                  >
                    <IconChevronRight
                      className={cn(
                        "mt-0.5 size-4 flex-shrink-0 text-muted-foreground transition-transform",
                        !isCollapsed && "rotate-90"
                      )}
                    />
                    <div className="flex flex-1 flex-col gap-1">
                      <div className="flex items-start justify-between gap-3 min-w-0">
                        <div className="flex flex-1 items-center gap-2 min-w-0 text-sm text-foreground">
                          <span
                            role="button"
                            tabIndex={0}
                            onClick={(event) =>
                              handleAuthorActivate(
                                event,
                                email.author_id ?? null
                              )
                            }
                            onKeyDown={(event) =>
                              handleAuthorActivate(
                                event,
                                email.author_id ?? null
                              )
                            }
                            className="font-semibold hover:underline"
                          >
                            {email.author_name ?? email.author_email}
                          </span>
                          {displaySubject ? (
                            <span
                              className="flex-1 truncate text-sm text-muted-foreground"
                              title={displaySubject}
                            >
                              {displaySubject}
                            </span>
                          ) : null}
                        </div>
                        <div className="flex shrink-0 items-center gap-2 text-xs text-muted-foreground whitespace-nowrap">
                          {isCollapsed && hiddenReplyCount > 0 ? (
                            <span>[{hiddenReplyCount} more]</span>
                          ) : null}
                          <span>{formatRelativeTime(email.date)}</span>
                        </div>
                      </div>
                    </div>
                  </button>

                  {!isCollapsed ? (
                    <div className="ml-6 mt-2 space-y-2">
                      {cleanBody ? <EmailBody body={cleanBody} /> : null}
                      {diffContent.trim().length > 0 ? (
                        <div className="max-w-full overflow-hidden">
                          <GitDiffViewer
                            diff={diffContent}
                            defaultExpanded={false}
                            gitCommitHash={email.git_commit_hash ?? null}
                          />
                        </div>
                      ) : null}
                      <div className="pt-2 text-xs text-muted-foreground">
                        {email.message_id}
                      </div>
                    </div>
                  ) : null}
                </div>
              </div>
            )
          })}
        </div>
      </ScrollArea>
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

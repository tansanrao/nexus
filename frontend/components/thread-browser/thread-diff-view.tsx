"use client"

import { IconGitBranch } from "@tabler/icons-react"
import { useMemo } from "react"

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Separator } from "@/components/ui/separator"
import { extractDiffContent } from "@/lib/diff"
import { formatDateTime } from "@/lib/locale-format"
import type { ThreadDetail } from "@src/lib/api"

import { GitDiffViewer } from "./git-diff-viewer"

type ThreadDiffViewProps = {
  selectedThreadId: string | null
  threadDetail: ThreadDetail | undefined
  isLoading: boolean
  error: unknown
  onShowConversation: () => void
}

export function ThreadDiffView({
  selectedThreadId,
  threadDetail,
  isLoading,
  error,
  onShowConversation,
}: ThreadDiffViewProps) {
  const { aggregatedDiff, includedEmails } = useMemo(() => {
    if (!threadDetail) {
      return { aggregatedDiff: "", includedEmails: [] as ThreadDetail["emails"] }
    }

    const emailsWithDiff = threadDetail.emails
      .map((email) => ({
        email,
        diff: extractDiffContent(email.body, email.patch_metadata),
      }))
      .filter(({ diff }) => diff && diff.trim().length > 0)

    if (emailsWithDiff.length === 0) {
      return { aggregatedDiff: "", includedEmails: [] as ThreadDetail["emails"] }
    }

    const combined = emailsWithDiff.map(({ diff }) => diff.trimEnd()).join("\n\n")

    return {
      aggregatedDiff: combined,
      includedEmails: emailsWithDiff.map(({ email }) => email),
    }
  }, [threadDetail])

  if (!selectedThreadId) {
    return (
      <div className="flex h-full items-center justify-center bg-muted/10">
        <p className="text-sm text-muted-foreground">
          Select a thread to view its diff summary.
        </p>
      </div>
    )
  }

  if (isLoading) {
    return (
      <div className="flex h-full flex-col gap-6 bg-background px-6 py-6">
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <IconGitBranch className="size-4" />
          Building diff…
        </div>
        {Array.from({ length: 4 }).map((_, index) => (
          <div key={index} className="h-20 animate-pulse rounded-md bg-muted/30" />
        ))}
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex h-full flex-col items-center justify-center bg-background px-6 py-6">
        <Alert variant="destructive" className="max-w-md">
          <AlertTitle>Unable to build diff</AlertTitle>
          <AlertDescription>
            {error instanceof Error ? error.message : "Something went wrong."}
          </AlertDescription>
        </Alert>
        <Button variant="ghost" className="mt-4" onClick={onShowConversation}>
          Back to conversation
        </Button>
      </div>
    )
  }

  if (!threadDetail) {
    return (
      <div className="flex h-full items-center justify-center bg-background px-6 py-6 text-sm text-muted-foreground">
        Thread details unavailable.
      </div>
    )
  }

  const { thread } = threadDetail

  return (
    <div className="flex h-full flex-col bg-background">
      <div className="flex flex-wrap items-start justify-between gap-3 border-b border-border px-6 py-5">
        <div className="space-y-2">
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <IconGitBranch className="size-4" />
            Combined git diff
          </div>
          <h2 className="text-lg font-semibold text-foreground">
            {thread.subject}
          </h2>
          <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
            <Badge variant="outline">
              {thread.message_count ?? 0} messages
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
        <Button variant="outline" size="sm" onClick={onShowConversation}>
          View conversation
        </Button>
      </div>

      <ScrollArea className="flex-1">
        <div className="flex flex-col gap-6 px-6 py-6">
          <div className="space-y-2 rounded-md border border-border bg-muted/10 px-4 py-4 text-sm text-muted-foreground">
            <p>
              {includedEmails.length} email
              {includedEmails.length === 1 ? "" : "s"} in this thread contained
              patch content. All patches are combined below.
            </p>
            {includedEmails.length > 0 ? (
              <div className="space-y-1 text-xs text-muted-foreground">
                <p className="font-medium text-foreground/80">Included messages</p>
                <ul className="list-disc list-inside space-y-0.5">
                  {includedEmails.map((email) => (
                    <li key={email.id}>
                      {email.subject || email.message_id}
                    </li>
                  ))}
                </ul>
              </div>
            ) : null}
          </div>
          <GitDiffViewer diff={aggregatedDiff} />
        </div>
      </ScrollArea>
    </div>
  )
}

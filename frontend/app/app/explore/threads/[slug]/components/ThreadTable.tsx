"use client"

import { IconLoader2 } from "@tabler/icons-react"
import { formatDistanceToNow } from "date-fns"

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { cn } from "@/lib/utils"
import type { ThreadWithStarter } from "@src/lib/api"

type ThreadTableProps = {
  threads: ThreadWithStarter[]
  isLoading?: boolean
}

export function ThreadTable({ threads, isLoading = false }: ThreadTableProps) {
  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12 text-muted-foreground">
        <IconLoader2 className="animate-spin mr-2" />
        Loading threads…
      </div>
    )
  }

  if (!threads.length) {
    return (
      <div className="rounded-xl border border-dashed p-8 text-muted-foreground">
        No threads found for this mailing list yet.
      </div>
    )
  }

  return (
    <div className="overflow-hidden rounded-xl border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead className="w-[40%]">Subject</TableHead>
            <TableHead className="min-w-[160px]">Starter</TableHead>
            <TableHead className="w-[160px]">Last Activity</TableHead>
            <TableHead className="w-[100px] text-right">Messages</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {threads.map((thread) => (
            <TableRow key={thread.id}>
              <TableCell>
                <div className="flex flex-col gap-1">
                  <span className="font-medium text-foreground line-clamp-2">
                    {thread.subject}
                  </span>
                  <span className="text-xs text-muted-foreground">
                    Root message: {thread.root_message_id}
                  </span>
                </div>
              </TableCell>
              <TableCell>
                <div className="flex flex-col text-sm">
                  <span className="text-foreground font-medium">
                    {thread.starter_name ?? thread.starter_email}
                  </span>
                  <span className="text-xs text-muted-foreground">
                    {thread.starter_email}
                  </span>
                </div>
              </TableCell>
              <TableCell className="text-sm text-muted-foreground">
                {formatRelative(thread.last_date)}
              </TableCell>
              <TableCell className="text-right">
                <span
                  className={cn(
                    "inline-flex items-center justify-end rounded-md border px-2 py-1 text-xs font-medium",
                    "border-muted-foreground/10 text-muted-foreground"
                  )}
                >
                  {thread.message_count ?? "—"}
                </span>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}

function formatRelative(date: string | null | undefined) {
  if (!date) return "Unknown"
  try {
    return formatDistanceToNow(new Date(date), { addSuffix: true })
  } catch {
    return new Date(date).toLocaleString()
  }
}

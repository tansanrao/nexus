import Link from "next/link"

import { AppPageHeader } from "@/components/layouts/app-page-header"
import { Button } from "@/components/ui/button"

export default function ExploreThreadsIndexPage() {
  return (
    <div className="flex h-full w-full flex-col overflow-auto">
      <AppPageHeader
        items={[
          {
            type: "link",
            label: "Explore",
            href: "/app/explore",
            hideOnMobile: true,
          },
          { type: "page", label: "Threads" },
        ]}
      />
      <div className="flex flex-1 min-h-0 flex-col gap-4 px-4 py-4">
        <div className="rounded-xl border border-dashed p-8 text-muted-foreground">
          <div className="mx-auto max-w-xl text-center space-y-4">
            <p>
              Start by picking a mailing list to explore. Use the slug selector on any thread view, e.g.
            </p>
            <div className="flex flex-wrap items-center justify-center gap-3">
              <Button asChild variant="outline" size="sm">
                <Link href="/app/explore/threads/lkml">Open lkml</Link>
              </Button>
              <Button asChild variant="outline" size="sm">
                <Link href="/app/explore/threads/bpf">Open bpf</Link>
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

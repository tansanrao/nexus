import Link from "next/link"

import { AppPageHeader } from "@/components/layouts/app-page-header"
import { Button } from "@/components/ui/button"

export default function ExploreThreadsIndexPage() {
  return (
    <div className="flex h-full flex-col">
      <AppPageHeader
        items={[
          { label: "Explore", href: "/app/explore", hideOnMobile: true },
          { label: "Threads" },
        ]}
      />
      <main className="flex flex-1 flex-col gap-4 p-4 pt-0">
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
      </main>
    </div>
  )
}

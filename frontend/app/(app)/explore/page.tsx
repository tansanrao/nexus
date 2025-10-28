import { AppPageHeader } from "@/components/layouts/app-page-header"

export default function ExplorePage() {
  return (
    <div className="flex h-full w-full flex-col overflow-auto">
      <AppPageHeader items={[{ type: "page", label: "Explore" }]} />
      <div className="flex flex-1 min-h-0 flex-col gap-4 px-4 py-4">
        <div className="rounded-xl border border-dashed p-8 text-muted-foreground">
          Start exploring Nexus data from the threads or authors views.
        </div>
      </div>
    </div>
  )
}

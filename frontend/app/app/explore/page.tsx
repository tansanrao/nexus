import { AppPageHeader } from "@/components/layouts/app-page-header"

export default function ExplorePage() {
  return (
    <>
      <AppPageHeader items={[{ label: "Explore" }]} />
      <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
        <div className="rounded-xl border border-dashed p-8 text-muted-foreground">
          Start exploring Nexus data from the threads or authors views.
        </div>
      </div>
    </>
  )
}

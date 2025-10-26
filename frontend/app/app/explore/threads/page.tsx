import { AppPageHeader } from "@/components/layouts/app-page-header"

export default function ExploreThreadsPage() {
  return (
    <>
      <AppPageHeader
        items={[
          { label: "Explore", href: "/app/explore", hideOnMobile: true },
          { label: "Threads" },
        ]}
      />
      <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
        <div className="rounded-xl border border-dashed p-8 text-muted-foreground">
          Threads listing placeholder. Surface trending and recent mail here.
        </div>
      </div>
    </>
  )
}

import { AppPageHeader } from "@/components/layouts/app-page-header"

export default function ExploreAuthorsPage() {
  return (
    <>
      <AppPageHeader
        items={[
          { label: "Explore", href: "/app/explore", hideOnMobile: true },
          { label: "Authors" },
        ]}
      />
      <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
        <div className="rounded-xl border border-dashed p-8 text-muted-foreground">
          Authors directory placeholder. Highlight active contributors here.
        </div>
      </div>
    </>
  )
}

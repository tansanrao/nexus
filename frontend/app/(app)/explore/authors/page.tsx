import { AppPageHeader } from "@/components/layouts/app-page-header"

export default function ExploreAuthorsPage() {
  return (
    <div className="flex h-full w-full flex-col overflow-auto">
      <AppPageHeader
        items={[
          {
            type: "link",
            label: "Explore",
            href: "/explore",
            hideOnMobile: true,
          },
          { type: "page", label: "Authors" },
        ]}
      />
      <div className="flex flex-1 min-h-0 flex-col gap-4 px-4 py-4">
        <div className="rounded-xl border border-dashed p-8 text-muted-foreground">
          Authors directory placeholder. Highlight active contributors here.
        </div>
      </div>
    </div>
  )
}

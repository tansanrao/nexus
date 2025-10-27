import { AppPageHeader } from "@/components/layouts/app-page-header"

export default function SettingsSearchPage() {
  return (
    <div className="flex h-full w-full flex-col overflow-auto">
      <AppPageHeader
        items={[
          {
            type: "link",
            label: "Settings",
            href: "/app/settings",
            hideOnMobile: true,
          },
          { type: "page", label: "Search" },
        ]}
      />
      <div className="flex flex-1 min-h-0 flex-col gap-4 px-4 py-4">
        <div className="rounded-xl border border-dashed p-8 text-muted-foreground">
          Search settings placeholder. Configure indexing and relevance here.
        </div>
      </div>
    </div>
  )
}

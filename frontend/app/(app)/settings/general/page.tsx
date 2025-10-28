import { AppPageHeader } from "@/components/layouts/app-page-header"

export default function SettingsGeneralPage() {
  return (
    <div className="flex h-full w-full flex-col overflow-auto">
      <AppPageHeader
        items={[
          {
            type: "link",
            label: "Settings",
            href: "/settings",
            hideOnMobile: true,
          },
          { type: "page", label: "General" },
        ]}
      />
      <div className="flex flex-1 min-h-0 flex-col gap-4 px-4 py-4">
        <div className="rounded-xl border border-dashed p-8 text-muted-foreground">
          General settings placeholder. Wire up organization preferences here.
        </div>
      </div>
    </div>
  )
}

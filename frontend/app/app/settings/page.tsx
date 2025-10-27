import { AppPageHeader } from "@/components/layouts/app-page-header"

export default function SettingsPage() {
  return (
    <div className="flex h-full w-full flex-col overflow-auto">
      <AppPageHeader items={[{ type: "page", label: "Settings" }]} />
      <div className="flex flex-1 min-h-0 flex-col gap-4 px-4 py-4">
        <div className="rounded-xl border border-dashed p-8 text-muted-foreground">
          Choose a settings category from the sidebar to configure Nexus.
        </div>
      </div>
    </div>
  )
}

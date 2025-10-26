import { AppPageHeader } from "@/components/layouts/app-page-header"

export default function SettingsPage() {
  return (
    <>
      <AppPageHeader items={[{ label: "Settings" }]} />
      <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
        <div className="rounded-xl border border-dashed p-8 text-muted-foreground">
          Choose a settings category from the sidebar to configure Nexus.
        </div>
      </div>
    </>
  )
}

import { AppPageHeader } from "@/components/layouts/app-page-header"

export default function SettingsGeneralPage() {
  return (
    <>
      <AppPageHeader
        items={[
          { label: "Settings", href: "/app/settings", hideOnMobile: true },
          { label: "General" },
        ]}
      />
      <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
        <div className="rounded-xl border border-dashed p-8 text-muted-foreground">
          General settings placeholder. Wire up organization preferences here.
        </div>
      </div>
    </>
  )
}

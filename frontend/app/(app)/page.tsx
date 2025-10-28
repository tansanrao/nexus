import Image from "next/image"

import { AppPageHeader } from "@/components/layouts/app-page-header"

export default function AppHomePage() {
  return (
    <div className="flex h-full w-full flex-col overflow-auto">
      <AppPageHeader items={[{ type: "page", label: "Home" }]} />
      <div className="flex flex-1 min-h-0 flex-col gap-4 px-4 py-4">
        <div className="flex flex-1 items-center justify-center rounded-xl bg-muted">
          <Image
            src="/logo.svg"
            alt="Nexus"
            width={144}
            height={144}
            className="h-24 w-24 md:h-36 md:w-36"
            priority
          />
        </div>
      </div>
    </div>
  )
}

import Image from "next/image"

import { AppPageHeader } from "@/components/layouts/app-page-header"

export default function AppHomePage() {
  return (
    <>
      <AppPageHeader items={[{ label: "Home" }]} />
      <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
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
    </>
  )
}

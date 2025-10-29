import { AppPageHeader } from "@/components/layouts/app-page-header"
import { MailingListDirectory } from "@/components/explore/mailing-list-directory"

export default function ExploreThreadsIndexPage() {
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
          { type: "page", label: "Threads" },
        ]}
      />
      <div className="flex flex-1 min-h-0 flex-col gap-4 px-4 py-4">
        <MailingListDirectory />
      </div>
    </div>
  )
}

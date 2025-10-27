import type { ReactNode } from "react"

import { AppLayoutShell } from "@/components/layouts/app-layout-shell"

export default function AppLayout({ children }: { children: ReactNode }) {
  return <AppLayoutShell>{children}</AppLayoutShell>
}

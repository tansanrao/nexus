"use client"

import * as React from "react"
import type { LucideIcon } from "lucide-react"

type BaseBreadcrumbItem = {
  hideOnMobile?: boolean
}

export type AppBreadcrumbLinkItem = BaseBreadcrumbItem & {
  type: "link"
  label: string
  href: string
}

export type AppBreadcrumbPageItem = BaseBreadcrumbItem & {
  type: "page"
  label: string
}

export type AppBreadcrumbDropdownOption = {
  id?: string
  label: string
  description?: string
  href?: string
  onSelect?: () => void
  disabled?: boolean
  isActive?: boolean
  icon?: LucideIcon
  shortcut?: string
}

export type AppBreadcrumbDropdownItem = BaseBreadcrumbItem & {
  type: "dropdown"
  label?: string
  display?: "label" | "ellipsis"
  items: AppBreadcrumbDropdownOption[]
  disabled?: boolean
  align?: "start" | "center" | "end"
  side?: "top" | "bottom" | "left" | "right"
}

export type AppBreadcrumbEllipsisItem = BaseBreadcrumbItem & {
  type: "ellipsis"
  items?: AppBreadcrumbDropdownOption[]
  align?: "start" | "center" | "end"
  side?: "top" | "bottom" | "left" | "right"
}

export type AppBreadcrumbItem =
  | AppBreadcrumbLinkItem
  | AppBreadcrumbPageItem
  | AppBreadcrumbDropdownItem
  | AppBreadcrumbEllipsisItem

type AppLayoutContextValue = {
  breadcrumbs: AppBreadcrumbItem[]
  setBreadcrumbs: (items: AppBreadcrumbItem[]) => void
  actions: React.ReactNode
  setActions: (content: React.ReactNode) => void
}

const AppLayoutContext = React.createContext<AppLayoutContextValue | null>(null)

export function AppLayoutProvider({
  children,
}: {
  children: React.ReactNode
}) {
  const [breadcrumbs, setBreadcrumbState] = React.useState<AppBreadcrumbItem[]>(
    []
  )
  const [actions, setActionsState] = React.useState<React.ReactNode>(null)

  const setBreadcrumbs = React.useCallback((items: AppBreadcrumbItem[]) => {
    setBreadcrumbState(items)
  }, [])

  const setActions = React.useCallback((content: React.ReactNode) => {
    setActionsState(content)
  }, [])

  const value = React.useMemo<AppLayoutContextValue>(
    () => ({
      breadcrumbs,
      setBreadcrumbs,
      actions,
      setActions,
    }),
    [actions, breadcrumbs, setActions, setBreadcrumbs]
  )

  return (
    <AppLayoutContext.Provider value={value}>
      {children}
    </AppLayoutContext.Provider>
  )
}

export function useAppLayout() {
  const context = React.useContext(AppLayoutContext)

  if (!context) {
    throw new Error("useAppLayout must be used within an AppLayoutProvider")
  }

  return context
}

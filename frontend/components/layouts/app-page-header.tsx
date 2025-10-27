"use client"

import * as React from "react"

import {
  useAppLayout,
  type AppBreadcrumbItem,
} from "@/components/layouts/app-layout-context"

type AppPageHeaderProps = {
  items: AppBreadcrumbItem[]
  actions?: React.ReactNode
}

export function AppPageHeader({ items, actions }: AppPageHeaderProps) {
  const { setBreadcrumbs, setActions } = useAppLayout()

  React.useEffect(() => {
    setBreadcrumbs(items)
    setActions(actions ?? null)

    return () => {
      setBreadcrumbs([])
      setActions(null)
    }
  }, [actions, items, setActions, setBreadcrumbs])

  return null
}

export type {
  AppBreadcrumbItem,
  AppBreadcrumbDropdownItem,
  AppBreadcrumbDropdownOption,
  AppBreadcrumbEllipsisItem,
  AppBreadcrumbLinkItem,
  AppBreadcrumbPageItem,
} from "@/components/layouts/app-layout-context"

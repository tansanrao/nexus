"use client"

import * as React from "react"
import Link from "next/link"
import { useRouter } from "next/navigation"
import { Check, ChevronDown } from "lucide-react"

import { AppSidebar } from "@/components/layouts/app-sidebar"
import {
  AppLayoutProvider,
  type AppBreadcrumbDropdownItem,
  type AppBreadcrumbDropdownOption,
  type AppBreadcrumbEllipsisItem,
  type AppBreadcrumbItem,
  useAppLayout,
} from "@/components/layouts/app-layout-context"
import {
  Breadcrumb,
  BreadcrumbEllipsis,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Separator } from "@/components/ui/separator"
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar"
import { cn } from "@/lib/utils"

function AppLayoutHeader() {
  const { breadcrumbs, actions } = useAppLayout()

  return (
    <header className="flex h-16 shrink-0 items-center gap-2 border-b border-border bg-background/95 px-4 transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-12">
      <SidebarTrigger className="-ml-1" />
      <Separator orientation="vertical" className="mr-2 h-4" />
      <AppBreadcrumbs items={breadcrumbs} />
      {actions ? (
        <div className="ml-auto flex items-center gap-2">{actions}</div>
      ) : null}
    </header>
  )
}

function AppBreadcrumbs({ items }: { items: AppBreadcrumbItem[] }) {
  const router = useRouter()

  const handleOptionSelect = React.useCallback(
    (option: AppBreadcrumbDropdownOption) => {
      option.onSelect?.()
      if (option.href) {
        router.push(option.href)
      }
    },
    [router]
  )

  if (!items.length) {
    return null
  }

  const renderDropdownTrigger = (
    item: AppBreadcrumbDropdownItem,
    variant: "label" | "ellipsis"
  ) => {
    if (variant === "ellipsis") {
      return (
        <DropdownMenuTrigger
          className="flex size-8 items-center justify-center rounded-md p-1 text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50"
          disabled={item.disabled}
        >
          <BreadcrumbEllipsis className="size-4" />
          <span className="sr-only">Toggle menu</span>
        </DropdownMenuTrigger>
      )
    }

    const label = item.label ?? "Menu"

    return (
      <DropdownMenuTrigger
        className="flex h-8 items-center gap-1 rounded-md px-2 text-sm font-medium text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50"
        disabled={item.disabled}
      >
        <span className="max-w-[180px] truncate">{label}</span>
        <ChevronDown className="size-3.5" />
      </DropdownMenuTrigger>
    )
  }

  const renderDropdownItems = (options: AppBreadcrumbDropdownOption[]) => {
    if (!options.length) {
      return (
        <DropdownMenuItem disabled className="text-muted-foreground">
          No options
        </DropdownMenuItem>
      )
    }

    return options.map((option) => {
      const Icon = option.icon

      return (
        <DropdownMenuItem
          key={option.id ?? option.label}
          className={cn(
            "gap-2",
            option.isActive && "font-medium text-foreground"
          )}
          disabled={option.disabled}
          onSelect={(event) => {
            if (option.disabled) {
              event.preventDefault()
              return
            }
            handleOptionSelect(option)
          }}
        >
          {Icon ? <Icon className="size-4 shrink-0" /> : null}
          <div className="flex min-w-0 flex-1 flex-col">
            <span className="truncate">{option.label}</span>
            {option.description ? (
              <span className="text-xs text-muted-foreground truncate">
                {option.description}
              </span>
            ) : null}
          </div>
          {option.shortcut ? (
            <span className="text-xs text-muted-foreground">
              {option.shortcut}
            </span>
          ) : null}
          {option.isActive ? (
            <Check className="size-3.5 text-primary" />
          ) : null}
        </DropdownMenuItem>
      )
    })
  }

  const renderDropdown = (item: AppBreadcrumbDropdownItem) => {
    const variant = item.display ?? (item.label ? "label" : "ellipsis")

    return (
      <DropdownMenu>
        {renderDropdownTrigger(item, variant)}
        <DropdownMenuContent
          align={item.align}
          side={item.side}
          className="min-w-[10rem]"
        >
          {renderDropdownItems(item.items)}
        </DropdownMenuContent>
      </DropdownMenu>
    )
  }

  const renderEllipsis = (item: AppBreadcrumbEllipsisItem) => {
    if (item.items && item.items.length > 0) {
      return (
        <DropdownMenu>
          <DropdownMenuTrigger
            className="flex size-8 items-center justify-center rounded-md p-1 text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          >
            <BreadcrumbEllipsis className="size-4" />
            <span className="sr-only">Toggle menu</span>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            align={item.align}
            side={item.side}
            className="min-w-[10rem]"
          >
            {renderDropdownItems(item.items)}
          </DropdownMenuContent>
        </DropdownMenu>
      )
    }

    return <BreadcrumbEllipsis className="size-4 text-muted-foreground" />
  }

  const getBreadcrumbKey = (item: AppBreadcrumbItem, index: number) => {
    switch (item.type) {
      case "link":
      case "page":
        return `${item.type}-${item.label}`
      case "dropdown":
        return `dropdown-${item.label ?? index}`
      case "ellipsis":
        return `ellipsis-${index}`
    }
  }

  const renderItemContent = (item: AppBreadcrumbItem) => {
    switch (item.type) {
      case "link":
        return (
          <BreadcrumbLink asChild>
            <Link href={item.href}>{item.label}</Link>
          </BreadcrumbLink>
        )
      case "page":
        return <BreadcrumbPage>{item.label}</BreadcrumbPage>
      case "dropdown":
        return renderDropdown(item)
      case "ellipsis":
        return renderEllipsis(item)
    }

    return null
  }

  return (
    <Breadcrumb>
      <BreadcrumbList>
        {items.map((item, index) => {
          const isLast = index === items.length - 1
          const visibilityClass = item.hideOnMobile ? "hidden md:block" : undefined

          return (
            <React.Fragment key={getBreadcrumbKey(item, index)}>
              <BreadcrumbItem className={visibilityClass}>
                {renderItemContent(item)}
              </BreadcrumbItem>
              {!isLast ? (
                <BreadcrumbSeparator className={visibilityClass} />
              ) : null}
            </React.Fragment>
          )
        })}
      </BreadcrumbList>
    </Breadcrumb>
  )
}

export function AppLayoutShell({ children }: { children: React.ReactNode }) {
  return (
    <AppLayoutProvider>
      <SidebarProvider className="group/sidebar-wrapper flex h-dvh w-full min-h-0 overflow-hidden bg-background text-foreground">
        <AppSidebar />
        <SidebarInset className="flex h-full min-h-0 flex-1 flex-col overflow-hidden">
          <div className="flex h-full w-full min-h-0 flex-1 flex-col">
            <AppLayoutHeader />
            <main className="flex flex-1 min-h-0 min-w-0 overflow-hidden">
              {children}
            </main>
          </div>
        </SidebarInset>
      </SidebarProvider>
    </AppLayoutProvider>
  )
}

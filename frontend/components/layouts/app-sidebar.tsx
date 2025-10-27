"use client"

import * as React from "react"
import Image from "next/image"
import Link from "next/link"
import { Compass, Home, Settings } from "lucide-react"

import { NavMain } from "@/components/layouts/nav-main"
import { NavUser } from "@/components/layouts/nav-user"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from "@/components/ui/sidebar"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { useDevMode } from "@src/providers/DevModeProvider"

// This is sample data.
const data = {
  user: {
    name: "shadcn",
    email: "m@example.com",
    avatar: "/avatars/shadcn.jpg",
  },
  navMain: [
    {
      title: "Home",
      url: "/app",
      icon: Home,
    },
    {
      title: "Explore",
      url: "/app/explore",
      icon: Compass,
      items: [
        {
          title: "Threads",
          url: "/app/explore/threads",
        },
        {
          title: "Authors",
          url: "/app/explore/authors",
        },
      ],
    },
    {
      title: "Settings",
      url: "/app/settings",
      icon: Settings,
      items: [
        {
          title: "General",
          url: "/app/settings/general",
        },
        {
          title: "Database",
          url: "/app/settings/database",
        },
        {
          title: "Search",
          url: "/app/settings/search",
        },
      ],
    },
  ],
}

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const { isDevMode, isReady, setDevMode } = useDevMode()

  return (
    <Sidebar collapsible="icon" className="group/sidebar" {...props}>
      <SidebarHeader>
        <SidebarMenuItem>
          <SidebarMenuButton asChild>
            <Link
              href="/app"
              aria-label="Nexus home"
            >
              <Image
                src="/favicon/favicon.svg"
                alt="Nexus"
                width={32}
                height={32}
                className="h-5 w-5 shrink-0 object-contain"
                priority
              />
              <span className="ml-2 text-sm font-bold">Nexus</span>
            </Link>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarHeader>
      <SidebarContent>
        <NavMain items={data.navMain} />
      </SidebarContent>
      <SidebarFooter className="gap-3">
        <SidebarMenuItem>
          <SidebarMenuButton asChild>
            <>
            <Switch
              id="sidebar-dev-mode"
              checked={isDevMode}
              onCheckedChange={(checked) => setDevMode(checked)}
              disabled={!isReady}
              aria-label="Toggle developer mode"
            />
            Dev Mode
            </>
          </SidebarMenuButton>
        </SidebarMenuItem>
        <NavUser user={data.user} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}

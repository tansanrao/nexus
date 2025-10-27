"use client"

import * as React from "react"

type ThreadBrowserLayoutProps = {
  toolbar?: React.ReactNode
  sidebar: React.ReactNode
  content: React.ReactNode
}

export function ThreadBrowserLayout({
  toolbar,
  sidebar,
  content,
}: ThreadBrowserLayoutProps) {
  return (
    <div className="flex h-full w-full flex-col">
      {toolbar ? (
        <div className="border-b border-border bg-background/60 px-4 py-2 backdrop-blur supports-[backdrop-filter]:bg-background/80">
          {toolbar}
        </div>
      ) : null}
      <div className="flex flex-1 flex-col overflow-hidden md:flex-row">
        <aside className="border-border/80 bg-muted/10 md:flex-none md:border-r md:bg-background">
          <div className="flex h-[360px] min-h-0 w-full flex-col overflow-hidden md:h-full md:w-[360px] md:min-w-[300px] md:max-w-[420px]">
            {sidebar}
          </div>
        </aside>
        <section className="flex-1 min-w-0 overflow-hidden bg-background">
          {content}
        </section>
      </div>
    </div>
  )
}

"use client"

import { useEffect, useState } from "react"

import { useSyncShikiTheme } from "./use-sync-shiki-theme"
import { highlightAgent } from "@/lib/shiki"

type DiffCodeBlockProps = {
  code: string
}

export function DiffCodeBlock({ code }: DiffCodeBlockProps) {
  const [html, setHtml] = useState<string>("")
  useSyncShikiTheme()

  useEffect(() => {
    let cancelled = false
    const run = async () => {
      const highlighted = await highlightAgent.highlight({
        code,
        lang: "diff",
      })
      if (!cancelled) {
        setHtml(highlighted)
      }
    }
    void run()
    return () => {
      cancelled = true
    }
  }, [code])

  if (!code.trim()) {
    return null
  }

  return (
    <div
      className="overflow-x-auto rounded-md border border-border bg-muted/40 text-sm shadow-sm"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  )
}

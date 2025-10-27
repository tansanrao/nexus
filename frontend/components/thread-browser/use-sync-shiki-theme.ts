"use client"

import { useEffect } from "react"
import { useTheme } from "next-themes"

import { highlightAgent } from "@/lib/shiki"

const THEME_MAP: Record<string, "github-light" | "github-dark"> = {
  light: "github-light",
  dark: "github-dark",
}

export function useSyncShikiTheme() {
  const { resolvedTheme } = useTheme()

  useEffect(() => {
    const target =
      (resolvedTheme && THEME_MAP[resolvedTheme]) || "github-light"
    void highlightAgent.setTheme(target)
  }, [resolvedTheme])
}

"use client"

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react"

type DevModeContextValue = {
  isDevMode: boolean
  isReady: boolean
  setDevMode: (next: boolean) => void
  toggleDevMode: () => void
}

const storageKey = "nexus.devMode"
const isDevEnvironment = process.env.NODE_ENV !== "production"

const DevModeContext = createContext<DevModeContextValue | undefined>(undefined)

export function DevModeProvider({ children }: { children: ReactNode }) {
  const [isDevMode, setIsDevMode] = useState(() => isDevEnvironment)
  const [isReady, setIsReady] = useState(false)

  useEffect(() => {
    try {
      const storedValue =
        typeof window !== "undefined" ? window.localStorage.getItem(storageKey) : null
      if (storedValue !== null) {
        setIsDevMode(storedValue === "true")
      }
    } catch (error) {
      console.warn("Failed to read dev mode flag", error)
    } finally {
      setIsReady(true)
    }
  }, [])

  useEffect(() => {
    if (!isReady) {
      return
    }

    try {
      if (typeof window !== "undefined") {
        window.localStorage.setItem(storageKey, isDevMode ? "true" : "false")
      }
    } catch (error) {
      console.warn("Failed to persist dev mode flag", error)
    }
  }, [isDevMode, isReady])

  const setDevMode = useCallback((next: boolean) => {
    setIsDevMode(next)
  }, [])

  const toggleDevMode = useCallback(() => {
    setIsDevMode((prev) => !prev)
  }, [])

  const value = useMemo<DevModeContextValue>(
    () => ({ isDevMode, isReady, setDevMode, toggleDevMode }),
    [isDevMode, isReady, setDevMode, toggleDevMode]
  )

  return <DevModeContext.Provider value={value}>{children}</DevModeContext.Provider>
}

export function useDevMode() {
  const context = useContext(DevModeContext)
  if (!context) {
    throw new Error("useDevMode must be used within a DevModeProvider")
  }
  return context
}

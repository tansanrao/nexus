"use client"

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react"

import {
  configureAuth,
  clearAuth,
} from "@src/lib/api/http"
import {
  getSession,
  login as loginRequest,
  logout as logoutRequest,
  refreshSession,
} from "@src/lib/api/auth"
import type {
  LoginRequest,
  LoginResponse,
  LogoutRequest,
  RefreshResponse,
  UserSummary,
} from "@src/lib/api"

type AuthStatus = "loading" | "authenticated" | "unauthenticated"

type StoredAuthState = {
  accessToken: string | null
  accessTokenExpiresAt: string | null
  refreshTokenExpiresAt: string | null
  csrfToken: string | null
  user: UserSummary | null
}

type AuthState = StoredAuthState & {
  status: AuthStatus
}

type AuthContextValue = {
  status: AuthStatus
  user: UserSummary | null
  accessToken: string | null
  csrfToken: string | null
  login: (credentials: LoginRequest) => Promise<LoginResponse>
  logout: (request?: LogoutRequest) => Promise<void>
  refresh: () => Promise<boolean>
}

const STORAGE_KEY = "nexus-auth-state"
const EXPIRY_SKEW_MS = 30_000

const initialState: AuthState = {
  status: "loading",
  accessToken: null,
  accessTokenExpiresAt: null,
  refreshTokenExpiresAt: null,
  csrfToken: null,
  user: null,
}

const AuthContext = createContext<AuthContextValue | undefined>(undefined)

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<AuthState>(initialState)
  const stateRef = useRef<AuthState>(initialState)

  useEffect(() => {
    stateRef.current = state
  }, [state])

  const persistState = useCallback((stored: StoredAuthState) => {
    if (typeof window === "undefined") {
      return
    }
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(stored))
  }, [])

  const clearStoredState = useCallback(() => {
    if (typeof window === "undefined") {
      return
    }
    window.localStorage.removeItem(STORAGE_KEY)
  }, [])

  const applyTokens = useCallback(
    (payload: {
      accessToken: string
      accessTokenExpiresAt: string
      refreshTokenExpiresAt: string
      csrfToken: string
      user?: UserSummary | null
    }) => {
      setState((previous) => {
        const stored: StoredAuthState = {
          accessToken: payload.accessToken,
          accessTokenExpiresAt: payload.accessTokenExpiresAt,
          refreshTokenExpiresAt: payload.refreshTokenExpiresAt,
          csrfToken: payload.csrfToken,
          user: payload.user ?? previous.user,
        }

        const next: AuthState = {
          status: "authenticated",
          ...stored,
        }

        persistState(stored)
        return next
      })
    },
    [persistState]
  )

  const clearState = useCallback(() => {
    clearStoredState()
    setState({
      status: "unauthenticated",
      accessToken: null,
      accessTokenExpiresAt: null,
      refreshTokenExpiresAt: null,
      csrfToken: null,
      user: null,
    })
  }, [clearStoredState])

  const refresh = useCallback(async () => {
    try {
      const response: RefreshResponse = await refreshSession()
      applyTokens({
        accessToken: response.access_token,
        accessTokenExpiresAt: response.access_token_expires_at,
        refreshTokenExpiresAt: response.refresh_token_expires_at,
        csrfToken: response.csrf_token,
      })
      return true
    } catch {
      clearState()
      return false
    }
  }, [applyTokens, clearState])

  const login = useCallback(
    async (credentials: LoginRequest) => {
      const response = await loginRequest(credentials)
      applyTokens({
        accessToken: response.access_token,
        accessTokenExpiresAt: response.access_token_expires_at,
        refreshTokenExpiresAt: response.refresh_token_expires_at,
        csrfToken: response.csrf_token,
        user: response.user,
      })
      return response
    },
    [applyTokens]
  )

  const logout = useCallback(
    async (request?: LogoutRequest) => {
      try {
        await logoutRequest(request)
      } finally {
        clearState()
      }
    },
    [clearState]
  )

  const loadStoredState = useCallback((): StoredAuthState | null => {
    if (typeof window === "undefined") {
      return null
    }
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (!raw) {
      return null
    }
    try {
      return JSON.parse(raw) as StoredAuthState
    } catch {
      return null
    }
  }, [])

  const initialize = useCallback(async () => {
    const stored = loadStoredState()
    if (stored && !isExpired(stored.accessTokenExpiresAt)) {
      setState({
        status: "authenticated",
        ...stored,
      })
      return
    }

    if (stored && isExpired(stored.refreshTokenExpiresAt)) {
      clearStoredState()
      setState((prev) => ({
        ...prev,
        status: "unauthenticated",
        accessToken: null,
        accessTokenExpiresAt: null,
        refreshTokenExpiresAt: null,
        csrfToken: null,
        user: null,
      }))
      return
    }

    const refreshed = await refresh()
    if (!refreshed) {
      clearStoredState()
      setState((prev) => ({
        ...prev,
        status: "unauthenticated",
      }))
      return
    }

    try {
      await getSession()
    } catch {
      // Ignore session fetch errors; state already reflects refresh result
    }
  }, [clearStoredState, loadStoredState, refresh])

  useEffect(() => {
    initialize()
  }, [initialize])

  useEffect(() => {
    configureAuth({
      getAccessToken: () => stateRef.current.accessToken,
      getCsrfToken: () => stateRef.current.csrfToken,
      refreshTokens: refresh,
      onUnauthorized: () => {
        clearState()
      },
    })
    return () => clearAuth()
  }, [clearState, refresh])

  const contextValue = useMemo<AuthContextValue>(
    () => ({
      status: state.status,
      user: state.user,
      accessToken: state.accessToken,
      csrfToken: state.csrfToken,
      login,
      logout,
      refresh,
    }),
    [login, logout, refresh, state.accessToken, state.csrfToken, state.status, state.user]
  )

  return <AuthContext.Provider value={contextValue}>{children}</AuthContext.Provider>
}

export function useAuth() {
  const context = useContext(AuthContext)
  if (!context) {
    throw new Error("useAuth must be used within an AuthProvider")
  }
  return context
}

function isExpired(expiresAt: string | null): boolean {
  if (!expiresAt) {
    return true
  }
  const expiry = new Date(expiresAt).getTime()
  return Number.isNaN(expiry) || expiry - EXPIRY_SKEW_MS <= Date.now()
}

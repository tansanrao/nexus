import ky, { HTTPError, KyInstance } from "ky"
import { resolveAdminBase, resolveBackendBase } from "./config"
import { buildSearchParams } from "./serialization"

export type ApiError = {
  status: number
  message: string
  code?: string
  details?: unknown
}

export type ValueProvider = () => string | Promise<string | null | undefined> | undefined | null

export type AuthHandlers = {
  getAccessToken?: ValueProvider
  getCsrfToken?: ValueProvider
  refreshTokens?: () => Promise<boolean>
  onUnauthorized?: () => void
}

type ApiClientKind = "public" | "admin"

type RequestOptions = {
  searchParams?: Record<string, unknown>
  json?: unknown
  signal?: AbortSignal
  headers?: Record<string, string>
  client?: ApiClientKind
}

let authHandlers: AuthHandlers = {}

export function configureAuth(handlers: AuthHandlers | null) {
  authHandlers = handlers ?? {}
}

export function clearAuth() {
  authHandlers = {}
}

const clients: Record<ApiClientKind, KyInstance> = {
  public: createClient(resolveBackendBase()),
  admin: createClient(resolveAdminBase()),
}

export function getClient(kind: ApiClientKind = "public") {
  return clients[kind]
}

export async function getJson<T>(path: string, options: RequestOptions = {}): Promise<T> {
  try {
    const searchParams = options.searchParams ? buildSearchParams(options.searchParams) : undefined
    return await getClient(options.client).get(path, {
      searchParams,
      signal: options.signal,
      headers: options.headers,
    }).json<T>()
  } catch (error) {
    throw await normalizeError(error)
  }
}

export async function postJson<T>(path: string, body?: unknown, options: RequestOptions = {}): Promise<T> {
  try {
    const searchParams = options.searchParams ? buildSearchParams(options.searchParams) : undefined
    return await getClient(options.client)
      .post(path, {
        json: body,
        searchParams,
        signal: options.signal,
        headers: options.headers,
      })
      .json<T>()
  } catch (error) {
    throw await normalizeError(error)
  }
}

export async function patchJson<T>(path: string, body?: unknown, options: RequestOptions = {}): Promise<T> {
  try {
    const searchParams = options.searchParams ? buildSearchParams(options.searchParams) : undefined
    return await getClient(options.client)
      .patch(path, {
        json: body,
        searchParams,
        signal: options.signal,
        headers: options.headers,
      })
      .json<T>()
  } catch (error) {
    throw await normalizeError(error)
  }
}

export async function del(path: string, options: RequestOptions = {}): Promise<void> {
  try {
    const searchParams = options.searchParams ? buildSearchParams(options.searchParams) : undefined
    await getClient(options.client).delete(path, {
      searchParams,
      signal: options.signal,
      headers: options.headers,
    })
  } catch (error) {
    throw await normalizeError(error)
  }
}

async function normalizeError(error: unknown): Promise<ApiError> {
  if (error instanceof HTTPError) {
    const { response } = error
    let payload: unknown = null
    try {
      payload = await response.clone().json()
    } catch {
      payload = null
    }

    const payloadObject = toErrorPayload(payload)
    const message =
      payloadObject?.message ||
      payloadObject?.error ||
      response.statusText ||
      "Unexpected error communicating with API"

    return {
      status: response.status,
      message,
      code: payloadObject?.code,
      details: payloadObject?.details ?? payload,
    }
  }

  if (error instanceof Error) {
    return {
      status: 0,
      message: error.message,
    }
  }

  return {
    status: 0,
    message: "Unknown error",
    details: error,
  }
}

export function isApiError(error: unknown): error is ApiError {
  return Boolean(error && typeof error === "object" && "status" in error && "message" in error)
}

type ErrorPayload = {
  message?: string
  error?: string
  code?: string
  details?: unknown
}

function toErrorPayload(value: unknown): ErrorPayload | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return null
  }
  return value as ErrorPayload
}

function createClient(prefixUrl: string): KyInstance {
  const client = ky.create({
    prefixUrl,
    credentials: "include",
    retry: 0,
    hooks: {
      beforeRequest: [
        async (request) => {
          request.headers.set("Accept", "application/json")
          if (!request.headers.get("Content-Type") && request.method !== "GET") {
            request.headers.set("Content-Type", "application/json")
          }

          if (authHandlers.getAccessToken) {
            const token = await authHandlers.getAccessToken()
            if (token) {
              request.headers.set("Authorization", `Bearer ${token}`)
            } else {
              request.headers.delete("Authorization")
            }
          }

          if (authHandlers.getCsrfToken && request.method !== "GET" && request.method !== "HEAD") {
            const csrf = await authHandlers.getCsrfToken()
            if (csrf) {
              request.headers.set("X-CSRF-Token", csrf)
            } else {
              request.headers.delete("X-CSRF-Token")
            }
          }
        },
      ],
      afterResponse: [
        async (request, options, response) => {
          if (response.status !== 401) {
            if (response.status === 401) {
              authHandlers.onUnauthorized?.()
            }
            return response
          }

          if (!authHandlers.refreshTokens) {
            authHandlers.onUnauthorized?.()
            return response
          }

          try {
            const refreshed = await authHandlers.refreshTokens()
            if (!refreshed) {
              authHandlers.onUnauthorized?.()
              return response
            }
          } catch {
            authHandlers.onUnauthorized?.()
            return response
          }

          const retryHeaders = mergeHeaders(request.headers, options.headers)

          if (authHandlers.getAccessToken) {
            const token = await authHandlers.getAccessToken()
            if (token) {
              retryHeaders.set("Authorization", `Bearer ${token}`)
            } else {
              retryHeaders.delete("Authorization")
            }
          }

          if (authHandlers.getCsrfToken && request.method !== "GET" && request.method !== "HEAD") {
            const csrf = await authHandlers.getCsrfToken()
            if (csrf) {
              retryHeaders.set("X-CSRF-Token", csrf)
            } else {
              retryHeaders.delete("X-CSRF-Token")
            }
          }

          return client(new Request(request), {
            ...options,
            headers: retryHeaders,
          })
        },
      ],
    },
  })

  return client
}

function mergeHeaders(base: Headers, override?: Headers | HeadersInit): Headers {
  const merged = new Headers(base)
  if (override) {
    const overrideHeaders = new Headers(override)
    overrideHeaders.forEach((value, key) => {
      merged.set(key, value)
    })
  }
  return merged
}

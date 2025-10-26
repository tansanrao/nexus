import ky, { HTTPError, KyInstance } from "ky"
import { buildEndpointUrl, resolveBackendBase } from "./config"
import { buildSearchParams } from "./serialization"

export type ApiError = {
  status: number
  message: string
  code?: string
  details?: unknown
}

export type TokenProvider = () => string | Promise<string> | undefined | null

type RequestOptions = {
  searchParams?: Record<string, unknown>
  json?: unknown
  signal?: AbortSignal
  headers?: Record<string, string>
}

let tokenProvider: TokenProvider | null = null

export function setTokenProvider(provider: TokenProvider | null) {
  tokenProvider = provider
}

export function clearTokenProvider() {
  tokenProvider = null
}

const apiClient: KyInstance = ky.create({
  prefixUrl: resolveBackendBase(),
  hooks: {
    beforeRequest: [
      async (request) => {
        request.headers.set("Accept", "application/json")
        if (!request.headers.get("Content-Type") && request.method !== "GET") {
          request.headers.set("Content-Type", "application/json")
        }

        if (tokenProvider) {
          const token = await tokenProvider()
          if (token) {
            request.headers.set("Authorization", `Bearer ${token}`)
          }
        }
      },
    ],
  },
  retry: 0,
})

export function getClient() {
  return apiClient
}

export async function getJson<T>(path: string, options: RequestOptions = {}): Promise<T> {
  try {
    const searchParams = options.searchParams ? buildSearchParams(options.searchParams) : undefined
    return await apiClient.get(path, { searchParams, signal: options.signal, headers: options.headers }).json<T>()
  } catch (error) {
    throw await normalizeError(error)
  }
}

export async function postJson<T>(path: string, body?: unknown, options: RequestOptions = {}): Promise<T> {
  try {
    const searchParams = options.searchParams ? buildSearchParams(options.searchParams) : undefined
    return await apiClient
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
    return await apiClient
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
    await apiClient.delete(path, { searchParams, signal: options.signal, headers: options.headers })
  } catch (error) {
    throw await normalizeError(error)
  }
}

export function withBase(path: string, base?: string) {
  return buildEndpointUrl(path, base)
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

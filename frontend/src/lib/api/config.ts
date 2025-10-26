const DEFAULT_BACKEND_PATH = "/api"
const API_VERSION_PATH = "/v1"

export const ApiEndpoints = {
  health: "health",
  mailingLists: "mailing-lists",
  threads: "threads",
  emails: "emails",
  authors: "authors",
  stats: "stats",
  admin: "admin",
} as const

export type ApiEndpoint = (typeof ApiEndpoints)[keyof typeof ApiEndpoints]

/**
 * Resolve the backend base URL, ensuring `/v1` is appended exactly once.
 */
export function resolveBackendBase(input?: string): string {
  const raw = (input ?? process.env.NEXT_PUBLIC_BACKEND_API_URL ?? DEFAULT_BACKEND_PATH).trim()
  const fallback = `${DEFAULT_BACKEND_PATH}${API_VERSION_PATH}`

  if (raw.length === 0) {
    return fallback
  }

  const withoutTrailingSlash = raw.replace(/\/+$/, "")
  const normalized =
    withoutTrailingSlash.startsWith("http://") || withoutTrailingSlash.startsWith("https://")
      ? withoutTrailingSlash
      : withoutTrailingSlash.startsWith("/")
        ? withoutTrailingSlash
        : `/${withoutTrailingSlash}`

  if (normalized.endsWith(API_VERSION_PATH)) {
    return normalized
  }

  return `${normalized}${API_VERSION_PATH}`
}

export function buildEndpointUrl(path: string, base?: string): string {
  const prefix = resolveBackendBase(base)
  const suffix = path.startsWith("/") ? path.slice(1) : path
  return `${prefix}/${suffix}`
}

export function getDefaultBackendPath() {
  return DEFAULT_BACKEND_PATH
}

const DEFAULT_BACKEND_PATH = "/api"
const DEFAULT_ADMIN_PATH = "/admin"
const API_VERSION_PATH = "/v1"

const BACKEND_ENV = process.env.NEXT_PUBLIC_BACKEND_API_URL
const ADMIN_ENV = process.env.NEXT_PUBLIC_ADMIN_API_URL

function normalizeBase(input: string, defaultPath: string) {
  const raw = input.trim()
  const fallback = `${defaultPath}${API_VERSION_PATH}`

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

export function resolveBackendBase(input?: string): string {
  return normalizeBase(input ?? BACKEND_ENV ?? DEFAULT_BACKEND_PATH, DEFAULT_BACKEND_PATH)
}

export function resolveAdminBase(input?: string): string {
  return normalizeBase(input ?? ADMIN_ENV ?? DEFAULT_ADMIN_PATH, DEFAULT_ADMIN_PATH)
}

export function getDefaultBackendPath() {
  return DEFAULT_BACKEND_PATH
}

export function getDefaultAdminPath() {
  return DEFAULT_ADMIN_PATH
}

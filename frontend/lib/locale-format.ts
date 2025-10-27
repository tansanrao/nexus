const relativeTimeFormatter =
  typeof Intl !== "undefined"
    ? new Intl.RelativeTimeFormat(undefined, { numeric: "auto" })
    : null

const dateTimeFormatter =
  typeof Intl !== "undefined"
    ? new Intl.DateTimeFormat(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "numeric",
        minute: "2-digit",
      })
    : null

const dateFormatter =
  typeof Intl !== "undefined"
    ? new Intl.DateTimeFormat(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
      })
    : null

function toDate(value: string | null | undefined): Date | null {
  if (!value) return null
  const timestamp = Date.parse(value)
  if (Number.isNaN(timestamp)) return null
  return new Date(timestamp)
}

export function formatRelativeTime(value: string | null | undefined): string {
  const date = toDate(value)
  if (!date || !relativeTimeFormatter) {
    return value ?? "Unknown"
  }

  const now = Date.now()
  const diffInSeconds = Math.round((date.getTime() - now) / 1000)

  const absSeconds = Math.abs(diffInSeconds)
  if (absSeconds < 45) {
    return relativeTimeFormatter.format(Math.round(diffInSeconds), "second")
  }
  const diffInMinutes = Math.round(diffInSeconds / 60)
  if (Math.abs(diffInMinutes) < 45) {
    return relativeTimeFormatter.format(diffInMinutes, "minute")
  }

  const diffInHours = Math.round(diffInSeconds / 3600)
  if (Math.abs(diffInHours) < 22) {
    return relativeTimeFormatter.format(diffInHours, "hour")
  }

  const diffInDays = Math.round(diffInSeconds / (3600 * 24))
  if (Math.abs(diffInDays) < 26) {
    return relativeTimeFormatter.format(diffInDays, "day")
  }

  const diffInMonths = Math.round(diffInSeconds / (3600 * 24 * 30))
  if (Math.abs(diffInMonths) < 11) {
    return relativeTimeFormatter.format(diffInMonths, "month")
  }

  const diffInYears = Math.round(diffInSeconds / (3600 * 24 * 365))
  return relativeTimeFormatter.format(diffInYears, "year")
}

export function formatDateTime(
  value: string | null | undefined
): string {
  const date = toDate(value)
  if (!date || !dateTimeFormatter) {
    return value ?? "Unknown"
  }
  return dateTimeFormatter.format(date)
}

export function formatDate(value: string | null | undefined): string {
  const date = toDate(value)
  if (!date || !dateFormatter) {
    return value ?? "Unknown"
  }
  return dateFormatter.format(date)
}

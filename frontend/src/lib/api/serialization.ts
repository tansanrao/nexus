type Primitive = string | number | boolean | null | undefined | Date
type QueryValue = Primitive | Primitive[] | Record<string, unknown>

function formatValue(value: Primitive): string | undefined {
  if (value === undefined || value === null) {
    return undefined
  }

  if (value instanceof Date) {
    return value.toISOString()
  }

  return String(value)
}

function append(entries: [string, string | string[]][], key: string, value: QueryValue) {
  if (Array.isArray(value)) {
    const formatted = value
      .map((item) => formatValue(item))
      .filter((item): item is string => typeof item === "string")
    if (formatted.length > 0) {
      entries.push([key, formatted])
    }
    return
  }

  if (value && typeof value === "object" && !(value instanceof Date)) {
    for (const [nestedKey, nestedValue] of Object.entries(value)) {
      append(entries, `${key}.${nestedKey}`, nestedValue as QueryValue)
    }
    return
  }

  const formatted = formatValue(value)
  if (formatted !== undefined) {
    entries.push([key, formatted])
  }
}

export function buildSearchParams(input: Record<string, unknown>) {
  const entries: [string, string | string[]][] = []
  for (const [key, value] of Object.entries(input)) {
    append(entries, key, value as QueryValue)
  }

  const params = new URLSearchParams()
  for (const [key, value] of entries) {
    if (Array.isArray(value)) {
      value.forEach((item) => params.append(key, item))
    } else {
      params.append(key, value)
    }
  }

  return params
}

#!/usr/bin/env tsx

/**
 * Fetch the OpenAPI spec from the running Nexus API and write it into docs/.
 */
import { mkdir, writeFile, cp } from "node:fs/promises"
import { dirname, resolve } from "node:path"
import { fileURLToPath } from "node:url"

type CliOptions = {
  url: string
  output: string
  snapshot: boolean
}

const DEFAULT_URL = process.env.OPENAPI_URL ?? "http://localhost:8000/api/v1/openapi.json"
const DEFAULT_OUTPUT = "docs/openapi-latest.json"

function parseArgs(argv: string[]): CliOptions {
  const options: CliOptions = {
    url: DEFAULT_URL,
    output: DEFAULT_OUTPUT,
    snapshot: true,
  }

  for (let i = 2; i < argv.length; i += 1) {
    const arg = argv[i]
    if (arg === "--url" && argv[i + 1]) {
      options.url = argv[i + 1]
      i += 1
    } else if (arg === "--output" && argv[i + 1]) {
      options.output = argv[i + 1]
      i += 1
    } else if (arg === "--no-snapshot") {
      options.snapshot = false
    } else {
      console.warn(`[fetch-openapi] Ignoring unknown argument: ${arg}`)
    }
  }

  return options
}

async function main() {
  const { url, output, snapshot } = parseArgs(process.argv)
  const start = performance.now()

  console.info(`[fetch-openapi] Fetching ${url}`)
  const response = await fetch(url)
  if (!response.ok) {
    throw new Error(`Failed to fetch OpenAPI spec (${response.status} ${response.statusText})`)
  }

  const spec = await response.text()
  const outPath = resolve(process.cwd(), output)

  await mkdir(dirname(outPath), { recursive: true })
  await writeFile(outPath, spec, "utf8")

  console.info(`[fetch-openapi] Wrote latest spec to ${outPath}`)

  if (snapshot) {
    const now = new Date()
    const stamp = now.toISOString().slice(0, 10).replace(/-/g, "")
    const snapshotPath = resolve(process.cwd(), `docs/openapi-${stamp}.json`)
    if (snapshotPath !== outPath) {
      await cp(outPath, snapshotPath, { force: true })
      console.info(`[fetch-openapi] Snapshot saved to ${snapshotPath}`)
    }
  }

  const elapsed = (performance.now() - start).toFixed(0)
  console.info(`[fetch-openapi] Done in ${elapsed}ms`)
}

main().catch((error) => {
  console.error(`[fetch-openapi] ${error instanceof Error ? error.message : String(error)}`)
  process.exitCode = 1
})

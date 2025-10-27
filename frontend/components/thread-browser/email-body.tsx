"use client"

import {
  useEffect,
  useMemo,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
  type MouseEvent as ReactMouseEvent,
} from "react"

import { cn } from "@/lib/utils"

type EmailBodyProps = {
  body: string
}

type QuoteSegment =
  | { type: "text"; lines: string[] }
  | { type: "quote"; node: QuoteNode }

type QuoteNode = {
  depth: number
  segments: QuoteSegment[]
}

type FormattedLine = {
  depth: number
  text: string
}

const QUOTE_STYLES = [
  { marker: "bg-sky-500/80", text: "text-sky-500 dark:text-sky-300" },
  { marker: "bg-emerald-500/80", text: "text-emerald-500 dark:text-emerald-300" },
  { marker: "bg-amber-500/80", text: "text-amber-500 dark:text-amber-300" },
  { marker: "bg-fuchsia-500/80", text: "text-fuchsia-500 dark:text-fuchsia-300" },
  { marker: "bg-purple-500/80", text: "text-purple-500 dark:text-purple-300" },
]

export function EmailBody({ body }: EmailBodyProps) {
  const parsed = useMemo(() => parseQuotedBody(body), [body])

  if (!body.trim()) {
    return null
  }

  return (
    <div className="max-w-full overflow-x-auto rounded-md bg-muted/20 p-3 text-sm font-mono leading-relaxed text-foreground">
      <div className="flex flex-col gap-0">
        <QuoteNodeRenderer node={parsed} />
      </div>
    </div>
  )
}

function QuoteNodeRenderer({ node }: { node: QuoteNode }) {
  let hasRenderedContent = false

  return (
    <>
      {node.segments.map((segment, index) => {
        if (segment.type === "text") {
          const displayLines = segment.lines.map((line) =>
            node.depth > 0 ? stripQuotePrefixForDepth(line, node.depth) : line
          )

          while (!hasRenderedContent && displayLines.length > 0 && displayLines[0].trim() === "") {
            displayLines.shift()
          }

          if (displayLines.length === 0) {
            return null
          }

          hasRenderedContent = true
          return <QuoteText key={index} lines={displayLines} depth={node.depth} />
        }

        hasRenderedContent = true
        return <QuoteBlock key={index} node={segment.node} />
      })}
    </>
  )
}

function QuoteText({ lines, depth }: { lines: string[]; depth: number }) {
  return (
    <>
      {lines.map((text, index) => (
        <EmailLine key={index} line={{ depth, text }} />
      ))}
    </>
  )
}

function QuoteBlock({ node }: { node: QuoteNode }) {
  const [collapsed, setCollapsed] = useState(false)
  const lineCount = useMemo(() => countQuoteLines(node), [node])

  useEffect(() => {
    setCollapsed(false)
  }, [node])

  const toggleCollapsed = (
    event: ReactMouseEvent<HTMLDivElement> | ReactKeyboardEvent<HTMLDivElement>
  ) => {
    const target = event.target as HTMLElement

    if (target.closest("a")) {
      event.stopPropagation()
      return
    }

    const selection =
      typeof window !== "undefined" ? window.getSelection() : null

    if (selection && selection.toString()) {
      event.stopPropagation()
      return
    }

    event.stopPropagation()
    setCollapsed((value) => !value)
  }

  const handleKeyDown = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault()
      toggleCollapsed(event)
    }
  }

  return (
    <div
      data-quote-block
      className="flex cursor-pointer flex-col gap-0 rounded-sm focus:outline-none focus-visible:ring-1 focus-visible:ring-ring focus-visible:ring-offset-0"
      onClick={toggleCollapsed}
      role="button"
      tabIndex={0}
      onKeyDown={handleKeyDown}
      aria-expanded={!collapsed}
      aria-label={collapsed ? "Expand quoted text" : "Collapse quoted text"}
    >
      {collapsed ? (
        <EmailLine
          line={{
            depth: node.depth,
            text: `[${lineCount} ${lineCount === 1 ? "line" : "lines"} hidden]`,
          }}
        />
      ) : (
        <div className="flex flex-col gap-0">
          <QuoteNodeRenderer node={node} />
        </div>
      )}
    </div>
  )
}

function EmailLine({ line }: { line: FormattedLine }) {
  const content = line.text === "" ? "\u00A0" : line.text
  const depthStyle =
    line.depth > 0 ? QUOTE_STYLES[(line.depth - 1) % QUOTE_STYLES.length] : null

  return (
    <div className="flex items-stretch gap-2">
      {line.depth > 0 && (
        <div className="flex gap-[2px] pr-1">
          {Array.from({ length: line.depth }).map((_, index) => {
            const style = QUOTE_STYLES[index % QUOTE_STYLES.length]
            return (
              <span
                key={index}
                className={cn("w-[3px] self-stretch rounded-sm", style.marker)}
              />
            )
          })}
        </div>
      )}
      <div
        className={cn(
          "flex-1 whitespace-pre-wrap break-words leading-relaxed",
          depthStyle ? ["pl-1", depthStyle.text] : "text-foreground"
        )}
      >
        {content}
      </div>
    </div>
  )
}

function parseQuotedBody(body: string): QuoteNode {
  const lines = body.split("\n")
  const rootNode: QuoteNode = { depth: 0, segments: [] }
  const stack: QuoteNode[] = [rootNode]

  const flushTextSegment = (node: QuoteNode, textLines: string[]) => {
    if (textLines.length === 0) {
      return
    }
    node.segments.push({ type: "text", lines: [...textLines] })
    textLines.length = 0
  }

  const createQuoteNode = (depth: number): QuoteNode => ({
    depth,
    segments: [],
  })

  let currentText: string[] = []

  for (const line of lines) {
    const depth = countQuoteDepth(line)
    const stackDepth = stack.length - 1

    if (depth > stackDepth) {
      const newNode = createQuoteNode(depth)
      stack[stack.length - 1].segments.push({ type: "quote", node: newNode })
      stack.push(newNode)
      currentText = []
    } else if (depth < stackDepth) {
      flushTextSegment(stack[stack.length - 1], currentText)
      while (stack.length - 1 > depth) {
        stack.pop()
      }
      currentText = []
    }

    const target = stack[stack.length - 1]
    currentText.push(line)
    flushTextSegment(target, currentText)
  }

  flushTextSegment(stack[stack.length - 1], currentText)

  return rootNode
}

function countQuoteDepth(line: string): number {
  let depth = 0
  while (depth < line.length && line[depth] === ">") {
    depth += 1
  }
  if (depth > 0 && line[depth] === " ") {
    return depth
  }
  return depth
}

function stripQuotePrefixForDepth(line: string, depth: number): string {
  let index = 0
  let seen = 0
  while (index < line.length && seen < depth) {
    if (line[index] === ">") {
      seen += 1
    }
    index += 1
    if (index < line.length && line[index] === " ") {
      index += 1
    }
  }
  return line.slice(index)
}

function countQuoteLines(node: QuoteNode): number {
  return node.segments.reduce((total, segment) => {
    if (segment.type === "text") {
      return total + segment.lines.length
    }
    return total + countQuoteLines(segment.node)
  }, 0)
}

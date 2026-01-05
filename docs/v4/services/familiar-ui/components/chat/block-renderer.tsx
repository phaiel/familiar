"use client"

import React from "react"
import { cn } from "@/lib/utils"
import type { 
  Block, 
  TextObject,
  PlainTextObject,
  ContextElement,
} from "@/types"

interface BlockRendererProps {
  blocks: Block[]
  className?: string
}

/**
 * BlockRenderer - Schema-first block rendering
 * 
 * Renders Block types generated from familiar-core Rust schemas.
 * All type definitions come from @familiar-core bindings.
 */
export function BlockRenderer({ blocks, className }: BlockRendererProps) {
  if (!blocks?.length) return null

  return (
    <div className={cn("flex flex-col gap-4 text-sm", className)}>
      {blocks.map((block, i) => (
        <BlockItem key={i} block={block} />
      ))}
    </div>
  )
}

function BlockItem({ block }: { block: Block }) {
  switch (block.type) {
    case "section":
      return (
        <div className="flex flex-col gap-2">
          <TextRenderer text={block.text} />
          {block.accessory && <div className="mt-2">{/* Accessory renderer - BlockElement */}</div>}
        </div>
      )
    case "header":
      return (
        <h3 className="font-semibold text-lg tracking-tight">
          <TextRenderer text={block.text} />
        </h3>
      )
    case "divider":
      return <hr className="my-2 border-border" />
    case "image":
      return (
        <div className="rounded-md overflow-hidden border border-border bg-muted/30">
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img src={block.image_url} alt={block.alt_text} className="w-full h-auto object-cover" />
        </div>
      )
    case "context":
      return (
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          {block.elements.map((el: ContextElement, i: number) => (
            <ContextElementRenderer key={i} element={el} />
          ))}
        </div>
      )
    case "accordion":
      return (
        <details className="group border border-border rounded-md bg-card" open={block.initial_state === "expanded"}>
          <summary className="flex items-center gap-2 p-3 font-medium cursor-pointer select-none">
            <span className="group-open:rotate-90 transition-transform">▶</span>
            <PlainTextRenderer text={block.summary} />
          </summary>
          <div className="p-3 pt-0 border-t border-border mt-2">
            <BlockRenderer blocks={block.blocks} />
          </div>
        </details>
      )
    case "actions":
      // Actions blocks contain interactive elements - placeholder for now
      return <div className="flex gap-2">{/* Actions block renderer */}</div>
    case "input":
      // Input blocks for forms - placeholder for now
      return <div>{/* Input block renderer */}</div>
    default:
      return null
  }
}

function ContextElementRenderer({ element }: { element: ContextElement }) {
  switch (element.type) {
    case "image":
      return (
        // eslint-disable-next-line @next/next/no-img-element
        <img src={element.image_url} alt={element.alt_text} className="w-4 h-4 rounded-sm" />
      )
    case "mrkdwn":
    case "plain_text":
      return <span>{element.text}</span>
    default:
      return null
  }
}

function TextRenderer({ text }: { text: TextObject }) {
  if (!text) return null
  // Both plain_text and mrkdwn have a `text` field
  return <span>{text.text}</span>
}

function PlainTextRenderer({ text }: { text: PlainTextObject }) {
  if (!text) return null
  return <span>{text.text}</span>
}


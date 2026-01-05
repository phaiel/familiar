import { useState, useCallback, useRef } from "react"
import type { Block, UIHeddleResult, MessageRole } from "@/types"

/**
 * ChatMessage - UI-specific chat message for the useChat hook
 * 
 * This is a UI-local type with presentation fields (isTyping, blocks).
 * For the database Message type, use `Message` from @/types.
 * For full agentic features, use chat-store.ts.
 */
export interface ChatMessage {
  id: string
  role: MessageRole
  content?: string
  blocks?: Block[]
  isTyping?: boolean
  meta?: {
    message_intent?: { intent: string }
    unit_count?: number
    heddle_result?: UIHeddleResult | null
    [key: string]: unknown
  }
}

export function useChat() {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [input, setInput] = useState("")
  const ws = useRef<WebSocket | null>(null)

  const addMessage = useCallback((msg: ChatMessage) => {
    setMessages((prev) => [...prev, msg])
  }, [])

  const updateMessage = useCallback((id: string, updates: Partial<ChatMessage>) => {
    setMessages((prev) => prev.map((m) => (m.id === id ? { ...m, ...updates } : m)))
  }, [])

  interface Attachment {
    file: File
    name: string
    type: string
    base64: string
  }

  const sendMessage = useCallback(async (text: string, attachments: Attachment[] = []) => {
    if (!text.trim() && attachments.length === 0) return

    const userMsgId = crypto.randomUUID()
    const assistantMsgId = crypto.randomUUID()

    // Add user message
    addMessage({
      id: userMsgId,
      role: "user",
      content: text,
    })

    setInput("")
    setIsLoading(true)

    // Add placeholder assistant message
    addMessage({
      id: assistantMsgId,
      role: "assistant",
      isTyping: true,
    })

    try {
      // Construct payload - aligns with WeaveRequest from familiar-core
      const payload: { weave?: string; blocks?: Array<{ type: string; [key: string]: unknown }> } = {}
      if (attachments.length > 0) {
        payload.blocks = [
          { type: "text", content: text },
          ...attachments.map(att => {
             // Convert attachments to blocks (simplified)
             if (att.type.startsWith('image')) return { type: 'image', source: att.base64, alt_text: att.name }
             return { type: 'document', source: att.base64, filename: att.name }
          })
        ]
      } else {
        payload.weave = text
      }

      const res = await fetch("/api/weave", { // Proxy should handle this
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      })

      const data = await res.json()

      if (data.job_id) {
        // Handle WS
        const wsUrl = `ws://localhost:3001${data.ws_url}`
        const socket = new WebSocket(wsUrl)
        ws.current = socket

        socket.onmessage = (event) => {
          const msg = JSON.parse(event.data)
          if (msg.type === "progress") {
             // Update progress
             updateMessage(assistantMsgId, { content: `Processing: ${msg.status}` })
          } else if (msg.type === "complete") {
             // Finalize
             if (msg.result) {
                 updateMessage(assistantMsgId, { 
                     isTyping: false,
                     content: msg.result.original_weave, // Or summary
                     blocks: msg.blocks, // If backend returns blocks
                     meta: msg.result
                 })
             }
             setIsLoading(false)
             socket.close()
          }
        }
      } else {
         // Sync response
         updateMessage(assistantMsgId, {
            isTyping: false,
            content: data.original_weave,
            meta: data
         })
         setIsLoading(false)
      }

    } catch (e) {
      console.error(e)
      updateMessage(assistantMsgId, { isTyping: false, content: "Error occurred." })
      setIsLoading(false)
    }
  }, [addMessage, updateMessage])

  return {
    messages,
    input,
    setInput,
    sendMessage,
    isLoading,
  }
}


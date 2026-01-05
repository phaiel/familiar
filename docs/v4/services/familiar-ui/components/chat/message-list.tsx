"use client"

import React from "react"
import { Message } from "@/hooks/use-chat"
import { BlockRenderer } from "./block-renderer"
/* eslint-disable @typescript-eslint/no-unused-vars */
import { Avatar, AvatarFallback } from "@/components/ui/avatar"
import { cn } from "@/lib/utils"

interface MessageListProps {
  messages: Message[]
  devMode: boolean
}

export function MessageList({ messages, devMode }: MessageListProps) {
  if (messages.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-muted-foreground p-8 text-center opacity-50">
        <div className="text-4xl mb-4">🧵</div>
        <h2 className="text-xl font-semibold">Welcome to Familiar</h2>
        <p className="mt-2 text-sm max-w-md">
          Start a conversation to see the Heddle Classification Engine in action.
          Upload images or audio for multimodal analysis.
        </p>
      </div>
    )
  }

  return (
    <div className="flex flex-col gap-6 py-4">
      {messages.map((msg) => (
        <MessageItem key={msg.id} message={msg} devMode={devMode} />
      ))}
      {/* Scroll anchor */}
      <div className="h-px" /> 
    </div>
  )
}

function MessageItem({ message, devMode }: { message: Message; devMode: boolean }) {
  const isUser = message.role === "user"

  return (
    <div className={cn("flex gap-3 max-w-3xl", isUser ? "ml-auto" : "mr-auto")}>
      {!isUser && (
        <Avatar className="h-8 w-8 border">
          <AvatarFallback>🤖</AvatarFallback>
        </Avatar>
      )}
      
      <div className={cn("flex flex-col gap-2 min-w-0", isUser ? "items-end" : "items-start")}>
        <div
          className={cn(
            "rounded-lg px-4 py-3 text-sm shadow-sm",
            isUser
              ? "bg-primary text-primary-foreground"
              : "bg-muted text-foreground border border-border"
          )}
        >
          {message.content && <div className="whitespace-pre-wrap">{message.content}</div>}
          
          {message.blocks && (
            <div className="mt-3 pt-3 border-t border-border/50">
               <BlockRenderer blocks={message.blocks} />
            </div>
          )}
          
          {message.isTyping && (
             <span className="flex gap-1 items-center mt-1 opacity-70">
                <span className="w-1.5 h-1.5 bg-current rounded-full animate-bounce" />
                <span className="w-1.5 h-1.5 bg-current rounded-full animate-bounce [animation-delay:0.1s]" />
                <span className="w-1.5 h-1.5 bg-current rounded-full animate-bounce [animation-delay:0.2s]" />
             </span>
          )}
        </div>

        {/* Dev Mode Details (only for assistant) */}
        {!isUser && devMode && message.meta && (
          <div className="w-full text-xs font-mono bg-card border rounded p-2 mt-1 overflow-x-auto max-w-xl">
             <div className="flex justify-between text-muted-foreground mb-1">
                <span>INTENT: {message.meta.message_intent?.intent}</span>
                <span>{message.meta.unit_count} Units</span>
             </div>
             <pre className="text-[10px] text-muted-foreground">
                {JSON.stringify(message.meta, null, 2)}
             </pre>
          </div>
        )}
      </div>

      {isUser && (
        <Avatar className="h-8 w-8 border bg-accent text-accent-foreground">
          <AvatarFallback>👤</AvatarFallback>
        </Avatar>
      )}
    </div>
  )
}


"use client"

import React, { useRef, useState } from "react"
import { useChat } from "@/hooks/use-chat"
import { PromptInput, PromptInputTextarea, PromptInputActions, PromptInputAction } from "@/components/ui/prompt-input"
import { Button } from "@/components/ui/button"
import { Paperclip, Mic, ArrowUp, Square } from "lucide-react"
import { MessageList } from "./message-list"
import { ScrollArea } from "@/components/ui/scroll-area"
import { DevToggle } from "./dev-toggle"
import { cn } from "@/lib/utils"

interface Attachment {
  file: File
  name: string
  type: string
  base64: string
}

export function ChatInterface() {
  const { messages, input, setInput, sendMessage, isLoading } = useChat()
  const [devMode, setDevMode] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [attachments, setAttachments] = useState<Attachment[]>([])

  const handleSubmit = () => {
    if (isLoading) return
    sendMessage(input, attachments)
    setAttachments([])
  }

  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      const newAttachments = Array.from(e.target.files).map(file => ({
        file,
        name: file.name,
        type: file.type,
        // In real impl, convert to base64 here
        base64: "data:image/png;base64,placeholder" 
      }))
      // Simplified: reading files would happen here
      // For now just mocking the structure
      const reader = new FileReader()
      reader.onload = (ev) => {
         newAttachments[0].base64 = ev.target?.result as string
         setAttachments([...attachments, ...newAttachments])
      }
      reader.readAsDataURL(e.target.files[0])
    }
  }

  return (
    <div className="flex flex-col h-screen max-w-4xl mx-auto p-4 gap-4">
      <header className="flex items-center justify-between py-2 border-b">
        <div className="flex items-center gap-2">
           <span className="text-xl">🧵</span>
           <h1 className="font-semibold text-lg">Familiar UI</h1>
        </div>
        <div className="flex items-center gap-4">
           <DevToggle enabled={devMode} onToggle={setDevMode} />
        </div>
      </header>

      <ScrollArea className="flex-1 pr-4">
        <MessageList messages={messages} devMode={devMode} />
      </ScrollArea>

      <div className="sticky bottom-0 bg-background pt-4">
        {attachments.length > 0 && (
          <div className="flex gap-2 mb-2 overflow-x-auto p-2 border rounded-md">
            {attachments.map((att, i) => (
              <div key={i} className="text-xs bg-muted p-1 rounded border flex items-center gap-1">
                <span>📎</span> {att.name}
              </div>
            ))}
          </div>
        )}
        
        <PromptInput
          value={input}
          onValueChange={setInput}
          onSubmit={handleSubmit}
          isLoading={isLoading}
          className="shadow-lg border-muted-foreground/20"
        >
          <PromptInputTextarea placeholder="How can I help you today?" />
          <PromptInputActions className="justify-between px-3 pb-3">
             <div className="flex gap-1">
                <PromptInputAction tooltip="Attach files">
                  <Button variant="ghost" size="icon" className="h-8 w-8 text-muted-foreground" onClick={() => fileInputRef.current?.click()}>
                    <Paperclip className="w-4 h-4" />
                    <span className="sr-only">Attach</span>
                  </Button>
                </PromptInputAction>
                <input 
                  type="file" 
                  multiple 
                  className="hidden" 
                  ref={fileInputRef} 
                  onChange={handleFileSelect}
                />
                <PromptInputAction tooltip="Voice input (Coming soon)">
                  <Button variant="ghost" size="icon" className="h-8 w-8 text-muted-foreground" disabled>
                    <Mic className="w-4 h-4" />
                    <span className="sr-only">Voice</span>
                  </Button>
                </PromptInputAction>
             </div>
             
             <Button 
                size="icon" 
                className={cn("rounded-full h-8 w-8", isLoading && "animate-pulse")} 
                onClick={handleSubmit}
                disabled={!input.trim() && attachments.length === 0}
             >
                {isLoading ? <Square className="w-4 h-4 fill-current" /> : <ArrowUp className="w-4 h-4" />}
                <span className="sr-only">Send</span>
             </Button>
          </PromptInputActions>
        </PromptInput>
        <div className="text-center text-xs text-muted-foreground mt-2 pb-2">
           Heddle Classification Engine Active • <span className="font-mono">v4.0</span>
        </div>
      </div>
    </div>
  )
}


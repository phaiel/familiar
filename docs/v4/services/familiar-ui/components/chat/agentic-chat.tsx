"use client"

import React, { useState, useEffect, useRef } from "react"
import { 
  ArrowUp, Bot, Hash, Plus, Loader2, ChevronDown, ChevronRight,
  Reply, Wrench, Brain, User, Send, X, FileText, Clock, Sparkles, CheckCircle,
  Database, Edit
} from "lucide-react"
import { cn } from "@/lib/utils"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Textarea } from "@/components/ui/textarea"
import { useChatStore, selectActiveChannel, isFamilyChannel, hasAIMention } from "@/stores/chat-store"
import type { 
  UIChannel, 
  UIChannelMessage, 
  UIThreadItem, 
  UIToolCall, 
  UIThinkingStep,
  UIHeddleResult,
  UIClassification,
  UIPhysicsResult,
  HeddleEntityType,
} from "@/types"

// ============================================================================
// Channel List (Left Panel)
// ============================================================================

function ChannelList() {
  const channels = useChatStore((s) => s.channels)
  const activeChannelId = useChatStore((s) => s.activeChannelId)
  const setActiveChannel = useChatStore((s) => s.setActiveChannel)
  const createChannel = useChatStore((s) => s.createChannel)
  const [newName, setNewName] = useState("")
  const [showNew, setShowNew] = useState(false)

  // Separate personal and family channels
  const personalChannels = channels.filter(ch => 
    ch.name.toLowerCase().includes('journal') || 
    ch.name.toLowerCase().includes('personal')
  )
  const familyChannels = channels.filter(ch => 
    !ch.name.toLowerCase().includes('journal') && 
    !ch.name.toLowerCase().includes('personal')
  )

  return (
    <div className="w-52 border-r flex flex-col bg-muted/20">
      <div className="p-2 border-b flex items-center justify-between">
        <span className="font-semibold text-sm">Channels</span>
        <Button variant="ghost" size="sm" className="h-6 w-6 p-0" onClick={() => setShowNew(!showNew)}>
          <Plus className="h-3 w-3" />
        </Button>
      </div>
      
      {showNew && (
        <div className="p-2 border-b flex gap-1">
          <input
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            placeholder="Name"
            className="flex-1 text-xs px-2 py-1 border rounded"
            onKeyDown={(e) => {
              if (e.key === "Enter" && newName.trim()) {
                createChannel(newName.trim())
                setNewName("")
                setShowNew(false)
              }
            }}
            autoFocus
          />
        </div>
      )}
      
      <ScrollArea className="flex-1">
        <div className="p-1 space-y-2">
          {/* Personal Channels */}
          {personalChannels.length > 0 && (
            <div>
              <div className="px-2 py-1 text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                Personal
              </div>
              {personalChannels.map((ch) => (
                <button
                  key={ch.id}
                  onClick={() => setActiveChannel(ch.id)}
                  className={cn(
                    "w-full text-left px-2 py-1.5 rounded text-xs flex items-center gap-1.5",
                    "hover:bg-muted transition-colors",
                    activeChannelId === ch.id && "bg-primary/10 font-medium border-l-2 border-primary"
                  )}
                >
                  <User className="h-3 w-3 text-primary flex-shrink-0" />
                  <span className="truncate">{ch.name}</span>
                  {ch.messages.length > 0 && (
                    <span className="ml-auto text-[10px] text-muted-foreground">{ch.messages.length}</span>
                  )}
                </button>
              ))}
            </div>
          )}

          {/* Family Channels */}
          {familyChannels.length > 0 && (
            <div>
              <div className="px-2 py-1 text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                Family
              </div>
              {familyChannels.map((ch) => (
                <button
                  key={ch.id}
                  onClick={() => setActiveChannel(ch.id)}
                  className={cn(
                    "w-full text-left px-2 py-1.5 rounded text-xs flex items-center gap-1.5",
                    "hover:bg-muted transition-colors",
                    activeChannelId === ch.id && "bg-muted font-medium"
                  )}
                >
                  <span className="text-sm flex-shrink-0">👨‍👩‍👧‍👦</span>
                  <span className="truncate">{ch.name}</span>
                  {ch.messages.length > 0 && (
                    <span className="ml-auto text-[10px] text-muted-foreground">{ch.messages.length}</span>
                  )}
                </button>
              ))}
            </div>
          )}

          {/* Fallback if no channels */}
          {channels.length === 0 && (
            <div className="px-2 py-4 text-xs text-center text-muted-foreground">
              No channels yet
            </div>
          )}
        </div>
      </ScrollArea>
    </div>
  )
}

// ============================================================================
// Thinking Indicator
// ============================================================================

function ThinkingIndicator({ activity }: { activity?: string | null }) {
  const [dots, setDots] = useState("")
  
  useEffect(() => {
    const interval = setInterval(() => {
      setDots((d) => (d.length >= 3 ? "" : d + "."))
    }, 400)
    return () => clearInterval(interval)
  }, [])

  return (
    <div className="flex items-center gap-2 text-sm text-muted-foreground animate-pulse">
      <Loader2 className="h-3 w-3 animate-spin" />
      <span>{activity || "Thinking"}{dots}</span>
    </div>
  )
}

// ============================================================================
// Chain of Thought Step (unified thinking + tool view)
// ============================================================================

interface ChainStep {
  id: string
  type: "thinking" | "tool_start" | "tool_complete" | "tool_error"
  agent?: string
  thought?: string
  tool?: string
  arguments?: unknown
  result?: unknown
  error?: string
  timestamp: string
}

function buildChainSteps(
  thinkingSteps: UIThinkingStep[] | undefined,
  toolCalls: UIToolCall[] | undefined
): ChainStep[] {
  const steps: ChainStep[] = []
  
  // Map thinking steps
  if (thinkingSteps) {
    for (const step of thinkingSteps) {
      // Check if this thinking step is about calling a tool
      const isToolCall = step.thought?.toLowerCase().includes('calling') || 
                        step.thought?.toLowerCase().includes('classifying') ||
                        step.thought?.toLowerCase().includes('computing') ||
                        step.thought?.toLowerCase().includes('determining') ||
                        step.thought?.toLowerCase().includes('searching') ||
                        step.thought?.toLowerCase().includes('generating')
      
      steps.push({
        id: step.id,
        type: "thinking",
        agent: step.agent || undefined,
        thought: step.thought || undefined,
        timestamp: step.timestamp || new Date().toISOString()
      })
    }
  }
  
  // Map tool calls with start and complete events
  if (toolCalls) {
    for (const tool of toolCalls) {
      // Tool completion (we show result after thinking step)
      steps.push({
        id: `${tool.id}-complete`,
        type: tool.status === "error" ? "tool_error" : "tool_complete",
        tool: tool.tool,
        arguments: tool.arguments,
        result: tool.result,
        error: (tool as UIToolCall & { error?: string }).error,
        timestamp: new Date().toISOString() // Tools don't have timestamps, place after thinking
      })
    }
  }
  
  return steps
}

function ChainStepItem({ step, isLast }: { step: ChainStep; isLast: boolean }) {
  const [showDetails, setShowDetails] = useState(false)
  
  if (step.type === "thinking") {
    // Determine if this is a tool-related thought
    const isToolThought = step.thought?.toLowerCase().includes('calling') || 
                          step.thought?.toLowerCase().includes('classifying') ||
                          step.thought?.toLowerCase().includes('computing') ||
                          step.thought?.toLowerCase().includes('determining') ||
                          step.thought?.toLowerCase().includes('searching') ||
                          step.thought?.toLowerCase().includes('generating') ||
                          step.thought?.toLowerCase().includes('analyzing')
    
    return (
      <div className="flex items-start gap-2 text-xs">
        <div className={cn(
          "w-5 h-5 rounded-full flex items-center justify-center flex-shrink-0",
          isToolThought ? "bg-blue-100 dark:bg-blue-900" : "bg-purple-100 dark:bg-purple-900"
        )}>
          {isToolThought ? (
            <Wrench className="h-2.5 w-2.5 text-blue-600" />
          ) : (
            <Brain className="h-2.5 w-2.5 text-purple-600" />
          )}
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-1.5">
            <span className="font-medium text-muted-foreground">{step.agent}</span>
            {isToolThought && <span className="text-blue-600 text-[10px]">→ tool</span>}
          </div>
          <p className="text-muted-foreground">{step.thought}</p>
        </div>
        {!isLast && <div className="w-px h-4 bg-border ml-2.5 -mb-2" />}
      </div>
    )
  }
  
  if (step.type === "tool_complete" || step.type === "tool_error") {
    const isError = step.type === "tool_error"
    
    // Format tool name nicely
    const toolName = step.tool?.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
    
    // Generate a human-readable summary of the result
    const getResultSummary = () => {
      if (!step.result) return null
      const result = step.result as Record<string, unknown>
      
      if (step.tool === "intent_classifier") {
        const intent = result.intent as string | undefined
        const confidence = result.confidence as number | undefined
        return `Intent: ${intent} (${Math.round((confidence || 0) * 100)}%)`
      }
      if (step.tool === "entity_classifier") {
        const classes = (result.classifications || []) as Array<{ entity_type: string; probability: number }>
        return classes.map((c) => `${c.entity_type} ${Math.round(c.probability * 100)}%`).join(", ")
      }
      if (step.tool === "physics_analyzer") {
        const valence = result.valence as number | undefined
        const arousal = result.arousal as number | undefined
        const epistemic = result.epistemic as number | undefined
        return `V:${valence?.toFixed(1)} A:${arousal?.toFixed(1)} E:${epistemic?.toFixed(1)}`
      }
      if (step.tool === "spawn_suggester") {
        const summary = (result.summary || {}) as { auto_spawn_count?: number; review_count?: number }
        return `${summary.auto_spawn_count || 0} auto, ${summary.review_count || 0} review`
      }
      return null
    }
    
    const summary = getResultSummary()
    
    return (
      <div className="flex items-start gap-2 text-xs">
        <div className={cn(
          "w-5 h-5 rounded-full flex items-center justify-center flex-shrink-0",
          isError ? "bg-red-100 dark:bg-red-900" : "bg-green-100 dark:bg-green-900"
        )}>
          {isError ? (
            <X className="h-2.5 w-2.5 text-red-600" />
          ) : (
            <CheckCircle className="h-2.5 w-2.5 text-green-600" />
          )}
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-1.5 flex-wrap">
            <span className={cn("font-medium", isError ? "text-red-600" : "text-green-700 dark:text-green-400")}>
              {toolName}
            </span>
            {summary && (
              <span className="text-muted-foreground bg-muted px-1.5 py-0.5 rounded text-[10px]">
                {summary}
              </span>
            )}
            <button 
              onClick={() => setShowDetails(!showDetails)}
              className="text-blue-600 hover:underline text-[10px]"
            >
              {showDetails ? "hide" : "details"}
            </button>
          </div>
          
          {isError && step.error && (
            <p className="text-red-600 mt-0.5">{step.error}</p>
          )}
          
          {showDetails && step.result != null && (
            <div className="mt-1.5 space-y-1">
              {step.arguments != null && (
                <div>
                  <span className="text-[10px] text-muted-foreground font-medium">INPUT:</span>
                  <pre className="p-1.5 bg-muted/50 rounded text-[10px] overflow-x-auto max-h-20">
                    {typeof step.arguments === 'string' ? step.arguments : JSON.stringify(step.arguments, null, 2)}
                  </pre>
                </div>
              )}
              <div>
                <span className="text-[10px] text-muted-foreground font-medium">OUTPUT:</span>
                <pre className="p-1.5 bg-green-50 dark:bg-green-950 rounded text-[10px] overflow-x-auto max-h-32">
                  {typeof step.result === 'string' ? step.result : JSON.stringify(step.result, null, 2)}
                </pre>
              </div>
            </div>
          )}
        </div>
        {!isLast && <div className="w-px h-4 bg-border ml-2.5 -mb-2" />}
      </div>
    )
  }
  
  return null
}

// ============================================================================
// Chain of Thought Section
// ============================================================================

function ChainOfThoughtSection({ 
  thinkingSteps, 
  toolCalls,
  defaultOpen = false,
  isLive = false
}: { 
  thinkingSteps: UIThinkingStep[] | undefined
  toolCalls: UIToolCall[] | undefined
  defaultOpen?: boolean
  isLive?: boolean
}) {
  const [open, setOpen] = useState(defaultOpen || isLive)
  
  // Auto-open when live and has content
  useEffect(() => {
    if (isLive && (thinkingSteps?.length || toolCalls?.length)) {
      setOpen(true)
    }
  }, [isLive, thinkingSteps?.length, toolCalls?.length])
  
  const steps = buildChainSteps(thinkingSteps, toolCalls)
  const toolCount = toolCalls?.length || 0
  const runningTools = toolCalls?.filter(t => t.status === "running").length || 0
  
  if (steps.length === 0) return null
  
  return (
    <div className={cn(
      "border-l-2 pl-2 py-1",
      isLive ? "border-purple-400 bg-purple-50/50 dark:bg-purple-950/20" : "border-purple-200"
    )}>
      <button
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1.5 text-xs hover:underline w-full text-left"
      >
        {open ? <ChevronDown className="h-3 w-3" /> : <ChevronRight className="h-3 w-3" />}
        {isLive ? (
          <Loader2 className="h-3 w-3 text-purple-500 animate-spin" />
        ) : (
          <Brain className="h-3 w-3 text-purple-500" />
        )}
        <span className={cn("font-medium", isLive ? "text-purple-600" : "text-purple-600")}>
          {isLive ? "Processing" : "Chain of Thought"}
        </span>
        {toolCount > 0 && (
          <>
            <span className="text-muted-foreground">•</span>
            <Wrench className="h-3 w-3 text-blue-500" />
            <span className="text-blue-600">{toolCount} tools</span>
            {runningTools > 0 && (
              <span className="text-orange-500 animate-pulse">({runningTools} running)</span>
            )}
          </>
        )}
        <span className="text-[10px] text-muted-foreground ml-1">({steps.length} steps)</span>
      </button>
      
      {open && (
        <div className="mt-2 ml-1 space-y-2">
          {steps.map((step, i) => (
            <ChainStepItem key={step.id} step={step} isLast={i === steps.length - 1} />
          ))}
          {isLive && (
            <div className="flex items-center gap-2 text-xs text-muted-foreground animate-pulse pl-7">
              <Loader2 className="h-2.5 w-2.5 animate-spin" />
              <span>Processing...</span>
            </div>
          )}
        </div>
      )}
    </div>
  )
}

// ============================================================================
// Heddle Result Section
// ============================================================================

// Extended Heddle result with spawn suggestions (extends UIHeddleResult)
interface ExtendedHeddleResult extends UIHeddleResult {
  intent?: { intent: string; confidence: number; pipeline?: string }
  spawn?: { 
    summary?: { auto_spawn_count?: number; review_count?: number }
    suggestions?: SpawnSuggestion[]
  }
  temporal_marker?: string
  weave_units?: WeaveUnitDisplay[]
}

interface SpawnSuggestion {
  entity_type: HeddleEntityType | string
  action: "auto_spawn" | "suggest" | "skip"
  confidence: number
  reason?: string
  content?: string
  physics?: PhysicsDisplay
  weave_unit_index?: number
}

interface WeaveUnitDisplay {
  content: string
  subject?: string
  unit_type?: string
  gathered_from?: string[]
  entities?: Array<{ type: string; probability: number }>
  physics?: PhysicsDisplay
  threads_referenced?: string[]
}

interface PhysicsDisplay {
  valence?: number
  arousal?: number
  significance?: number
  epistemic?: number
}

function HeddleResultSection({ heddle }: { heddle: ExtendedHeddleResult }) {
  const [open, setOpen] = useState(false)
  
  if (!heddle) return null
  
  // Quick summary for collapsed view
  const intent = heddle.intent?.intent || heddle.purpose || "Unknown"
  const confidence = heddle.intent?.confidence || 0
  const topEntity = heddle.classifications?.[0]
  const spawnCount = (heddle.spawn?.summary?.auto_spawn_count || 0) + (heddle.spawn?.summary?.review_count || 0)
  
  return (
    <div className="border-l-2 border-green-300 pl-2 py-1">
      <button
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1.5 text-xs hover:underline w-full text-left"
      >
        {open ? <ChevronDown className="h-3 w-3" /> : <ChevronRight className="h-3 w-3" />}
        <FileText className="h-3 w-3 text-green-600" />
        <span className="text-green-700 dark:text-green-400 font-medium">Heddle Analysis</span>
        <span className="text-muted-foreground">
          {intent} → {topEntity?.entity_type || "?"} 
          {spawnCount > 0 && ` • ${spawnCount} entities`}
        </span>
      </button>
      
      {open && (
        <div className="mt-2 ml-1 space-y-2 text-xs">
          {/* Intent */}
          <div className="flex items-center gap-2">
            <span className="text-muted-foreground w-16">Intent:</span>
            <span className="font-medium">{intent}</span>
            <span className="text-muted-foreground">({Math.round(confidence * 100)}%)</span>
            <span className="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded">
              {heddle.intent?.pipeline || "recording"}
            </span>
          </div>
          
          {/* Classifications */}
          {heddle.classifications?.length > 0 && (
            <div className="flex items-start gap-2">
              <span className="text-muted-foreground w-16">Entities:</span>
              <div className="flex flex-wrap gap-1">
                {heddle.classifications.map((c: UIClassification, i: number) => (
                  <span 
                    key={i} 
                    className={cn(
                      "px-1.5 py-0.5 rounded text-[10px]",
                      c.probability >= 0.8 ? "bg-green-100 dark:bg-green-900 text-green-700 dark:text-green-300" :
                      c.probability >= 0.5 ? "bg-yellow-100 dark:bg-yellow-900 text-yellow-700 dark:text-yellow-300" :
                      "bg-muted text-muted-foreground"
                    )}
                  >
                    {c.entity_type} {Math.round(c.probability * 100)}%
                  </span>
                ))}
              </div>
            </div>
          )}
          
          {/* Physics */}
          {heddle.physics && (
            <div className="flex items-center gap-2">
              <span className="text-muted-foreground w-16">Physics:</span>
              <div className="flex gap-2 text-[10px]">
                <span className={cn(
                  "px-1.5 py-0.5 rounded",
                  heddle.physics.valence > 0.3 ? "bg-green-100 text-green-700" :
                  heddle.physics.valence < -0.3 ? "bg-red-100 text-red-700" :
                  "bg-muted text-muted-foreground"
                )}>
                  V: {heddle.physics.valence?.toFixed(1)}
                </span>
                <span className={cn(
                  "px-1.5 py-0.5 rounded",
                  heddle.physics.arousal > 0.6 ? "bg-orange-100 text-orange-700" :
                  "bg-muted text-muted-foreground"
                )}>
                  A: {heddle.physics.arousal?.toFixed(1)}
                </span>
                <span className="px-1.5 py-0.5 rounded bg-muted text-muted-foreground">
                  E: {heddle.physics.epistemic?.toFixed(1)}
                </span>
                <span className={cn(
                  "px-1.5 py-0.5 rounded",
                  heddle.physics.significance > 0.7 ? "bg-purple-100 text-purple-700" :
                  "bg-muted text-muted-foreground"
                )}>
                  S: {heddle.physics.significance?.toFixed(1)}
                </span>
              </div>
            </div>
          )}
          
          {/* Spawn Suggestions */}
          {heddle.spawn?.suggestions?.length > 0 && (
            <div className="flex items-start gap-2">
              <span className="text-muted-foreground w-16">Spawn:</span>
              <div className="space-y-1">
                {heddle.spawn.suggestions.map((s: SpawnSuggestion, i: number) => (
                  <div key={i} className="flex items-center gap-1.5">
                    {s.action === "auto_spawn" ? (
                      <CheckCircle className="h-3 w-3 text-green-600" />
                    ) : s.action === "suggest" ? (
                      <Sparkles className="h-3 w-3 text-yellow-600" />
                    ) : (
                      <X className="h-3 w-3 text-muted-foreground" />
                    )}
                    <span className="font-medium">{s.entity_type}</span>
                    <span className="text-muted-foreground text-[10px]">
                      {s.action === "auto_spawn" ? "auto" : s.action === "suggest" ? "review" : "skip"}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
          
          {/* Temporal Marker */}
          {heddle.temporal_marker && (
            <div className="flex items-center gap-2">
              <span className="text-muted-foreground w-16">When:</span>
              <span className="text-[10px] bg-blue-100 dark:bg-blue-900 px-1.5 py-0.5 rounded text-blue-700 dark:text-blue-300">
                <Clock className="h-2.5 w-2.5 inline mr-1" />
                {heddle.temporal_marker}
              </span>
            </div>
          )}
        </div>
      )}
    </div>
  )
}

// ============================================================================
// Weave Units Section (Shows segmented content with classifications)
// ============================================================================

function WeaveUnitsSection({ weaveUnits }: { weaveUnits: WeaveUnitDisplay[] }) {
  const [open, setOpen] = useState(true)
  const [expandedUnits, setExpandedUnits] = useState<Set<number>>(new Set())
  
  if (!weaveUnits || weaveUnits.length === 0) return null
  
  const toggleExpanded = (i: number) => {
    setExpandedUnits(prev => {
      const next = new Set(prev)
      if (next.has(i)) next.delete(i)
      else next.add(i)
      return next
    })
  }
  
  // Group by subject for better organization
  const subjects = Array.from(new Set(weaveUnits.map(u => u.subject).filter(Boolean)))
  
  return (
    <div className="border-l-2 border-indigo-300 pl-2 py-1">
      <button
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1.5 text-xs hover:underline w-full text-left"
      >
        {open ? <ChevronDown className="h-3 w-3" /> : <ChevronRight className="h-3 w-3" />}
        <FileText className="h-3 w-3 text-indigo-600" />
        <span className="text-indigo-700 dark:text-indigo-400 font-medium">Weave Units</span>
        <span className="text-muted-foreground">
          ({weaveUnits.length} units
          {subjects.length > 0 && ` • ${subjects.length} subjects`})
        </span>
      </button>
      
      {open && (
        <div className="mt-2 space-y-2">
          {weaveUnits.map((unit: WeaveUnitDisplay, i: number) => {
            const isExpanded = expandedUnits.has(i)
            const hasGatheredFrom = unit.gathered_from?.length > 1
            
            return (
              <div 
                key={i} 
                className={cn(
                  "rounded-lg border p-2 text-xs transition-colors",
                  unit.unit_type === "evaluation" && "bg-pink-50/50 dark:bg-pink-950/20 border-pink-200",
                  unit.unit_type === "observation" && "bg-purple-50/50 dark:bg-purple-950/20 border-purple-200",
                  unit.unit_type === "event" && "bg-blue-50/50 dark:bg-blue-950/20 border-blue-200",
                  !["evaluation", "observation", "event"].includes(unit.unit_type) && "bg-white dark:bg-gray-900"
                )}
              >
                {/* Subject Header */}
                {unit.subject && (
                  <div className="flex items-center gap-1.5 mb-1">
                    <span className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wide">
                      {unit.subject}
                    </span>
                    {unit.unit_type && (
                      <span className="text-[9px] px-1 rounded bg-gray-100 dark:bg-gray-800 text-gray-500">
                        {unit.unit_type}
                      </span>
                    )}
                  </div>
                )}
                
                {/* Content */}
                <div className="font-medium text-foreground mb-1.5">"{unit.content}"</div>
                
                {/* Gathered From (semantic grouping visualization) */}
                {hasGatheredFrom && (
                  <div className="mb-1.5">
                    <button 
                      onClick={() => toggleExpanded(i)}
                      className="text-[10px] text-indigo-600 hover:underline flex items-center gap-0.5"
                    >
                      {isExpanded ? <ChevronDown className="h-2.5 w-2.5" /> : <ChevronRight className="h-2.5 w-2.5" />}
                      Gathered from {unit.gathered_from.length} parts
                    </button>
                    {isExpanded && (
                      <div className="mt-1 pl-3 border-l border-indigo-200 space-y-0.5">
                        {unit.gathered_from.map((part: string, j: number) => (
                          <div key={j} className="text-[10px] text-muted-foreground italic">
                            "{part}"
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                )}
                
                {/* Classifications */}
                <div className="flex flex-wrap gap-1 mb-1.5">
                  {(unit.entities || []).map((e: { type: string; probability: number }, j: number) => (
                    <span 
                      key={j}
                      className={cn(
                        "px-1.5 py-0.5 rounded text-[10px] font-medium",
                        e.type === "MOMENT" && "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300",
                        e.type === "PULSE" && "bg-pink-100 text-pink-700 dark:bg-pink-900 dark:text-pink-300",
                        e.type === "INTENT" && "bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300",
                        e.type === "THREAD" && "bg-cyan-100 text-cyan-700 dark:bg-cyan-900 dark:text-cyan-300",
                        e.type === "BOND" && "bg-amber-100 text-amber-700 dark:bg-amber-900 dark:text-amber-300",
                        !["MOMENT", "PULSE", "INTENT", "THREAD", "BOND"].includes(e.type) && "bg-gray-100 text-gray-700"
                      )}
                    >
                      {e.type} {Math.round((e.probability || 0) * 100)}%
                    </span>
                  ))}
                </div>
                
                {/* Physics */}
                {unit.physics && (
                  <div className="flex gap-2 text-[10px] text-muted-foreground">
                    <span className={cn(
                      "px-1 rounded",
                      unit.physics.valence > 0.3 ? "bg-green-100 text-green-600 dark:bg-green-900 dark:text-green-400" : 
                      unit.physics.valence < -0.3 ? "bg-red-100 text-red-600 dark:bg-red-900 dark:text-red-400" : "bg-gray-100"
                    )}>
                      V: {unit.physics.valence?.toFixed(1)}
                    </span>
                    <span className={cn(
                      "px-1 rounded",
                      unit.physics.arousal > 0.6 ? "bg-orange-100 text-orange-600" : "bg-gray-100"
                    )}>
                      A: {unit.physics.arousal?.toFixed(1)}
                    </span>
                    <span className={cn(
                      "px-1 rounded",
                      unit.physics.significance > 0.7 ? "bg-yellow-100 text-yellow-700" : "bg-gray-100"
                    )}>
                      S: {unit.physics.significance?.toFixed(1)}
                    </span>
                  </div>
                )}
                
                {/* Threads Referenced */}
                {unit.threads_referenced?.length > 0 && (
                  <div className="mt-1.5 flex items-center gap-1 text-[10px] text-muted-foreground">
                    <span>→</span>
                    {unit.threads_referenced.map((t: string, j: number) => (
                      <span key={j} className="px-1.5 bg-cyan-50 dark:bg-cyan-950 rounded text-cyan-600 dark:text-cyan-400">
                        {t}
                      </span>
                    ))}
                  </div>
                )}
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}

// ============================================================================
// Spawned Entities Section (HILT - Human in the Loop)
// ============================================================================

interface SpawnedEntity extends SpawnSuggestion {
  status?: "pending" | "approved" | "rejected" | "editing"
  edited_content?: string
}

function SpawnedEntitiesSection({ 
  suggestions, 
  content 
}: { 
  suggestions: SpawnSuggestion[]
  content: string 
}) {
  // Group suggestions by weave_unit_index for better organization
  const [entities, setEntities] = useState<SpawnedEntity[]>(() => 
    suggestions.map(s => ({
      ...s,
      status: s.action === "auto_spawn" ? "approved" : "pending"
    }))
  )
  const [editingId, setEditingId] = useState<number | null>(null)
  const [editText, setEditText] = useState("")
  const [open, setOpen] = useState(true)
  
  const pendingCount = entities.filter(e => e.status === "pending").length
  const approvedCount = entities.filter(e => e.status === "approved").length
  
  // Group by weave unit
  const unitGroups = entities.reduce((acc: Record<number, SpawnedEntity[]>, e) => {
    const key = e.weave_unit_index ?? -1
    if (!acc[key]) acc[key] = []
    acc[key].push(e)
    return acc
  }, {})
  
  const handleApprove = (index: number) => {
    setEntities(prev => prev.map((e, i) => 
      i === index ? { ...e, status: "approved" } : e
    ))
    console.log("HILT: Approved entity", entities[index])
  }
  
  const handleReject = (index: number) => {
    setEntities(prev => prev.map((e, i) => 
      i === index ? { ...e, status: "rejected" } : e
    ))
    console.log("HILT: Rejected entity", entities[index])
  }
  
  const handleEdit = (index: number) => {
    setEditingId(index)
    setEditText(entities[index].edited_content || entities[index].content || content)
  }
  
  const handleSaveEdit = (index: number) => {
    setEntities(prev => prev.map((e, i) => 
      i === index ? { ...e, edited_content: editText, status: "approved" } : e
    ))
    setEditingId(null)
    setEditText("")
    console.log("HILT: Saved edited entity", { ...entities[index], edited_content: editText })
  }
  
  const handleCancelEdit = () => {
    setEditingId(null)
    setEditText("")
  }
  
  const handleApproveAll = () => {
    setEntities(prev => prev.map(e => 
      e.status === "pending" ? { ...e, status: "approved" } : e
    ))
    console.log("HILT: Approved all pending entities")
  }
  
  const handleRejectAll = () => {
    setEntities(prev => prev.map(e => 
      e.status === "pending" ? { ...e, status: "rejected" } : e
    ))
    console.log("HILT: Rejected all pending entities")
  }
  
  return (
    <div className="border-l-2 border-orange-300 pl-2 py-1">
      <button
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1.5 text-xs hover:underline w-full text-left"
      >
        {open ? <ChevronDown className="h-3 w-3" /> : <ChevronRight className="h-3 w-3" />}
        <Database className="h-3 w-3 text-orange-600" />
        <span className="text-orange-700 dark:text-orange-400 font-medium">Spawn Suggestions</span>
        <span className="text-muted-foreground">
          {approvedCount} approved
          {pendingCount > 0 && <span className="text-orange-500"> • {pendingCount} pending</span>}
        </span>
      </button>
      
      {open && (
        <div className="mt-2 space-y-3">
          {/* Bulk Actions */}
          {pendingCount > 0 && (
            <div className="flex gap-2 pb-2 border-b">
              <Button 
                variant="outline" 
                size="sm" 
                className="h-6 text-[10px] text-green-600 border-green-300"
                onClick={handleApproveAll}
              >
                <CheckCircle className="h-3 w-3 mr-1" /> Approve All ({pendingCount})
              </Button>
              <Button 
                variant="outline" 
                size="sm" 
                className="h-6 text-[10px] text-red-600 border-red-300"
                onClick={handleRejectAll}
              >
                <X className="h-3 w-3 mr-1" /> Reject All
              </Button>
            </div>
          )}
          
          {/* Group by weave unit */}
          {Object.entries(unitGroups).sort(([a], [b]) => Number(a) - Number(b)).map(([unitIndex, unitEntities]) => (
            <div key={unitIndex} className="space-y-1.5">
              {Number(unitIndex) >= 0 && (
                <div className="text-[10px] text-muted-foreground font-medium">
                  Segment {Number(unitIndex) + 1}
                </div>
              )}
              {Number(unitIndex) === -1 && (
                <div className="text-[10px] text-muted-foreground font-medium">
                  Referenced Threads
                </div>
              )}
              
              {unitEntities.map((entity, localIdx) => {
                const globalIdx = entities.findIndex(e => e === entity)
                return (
                  <div 
                    key={globalIdx} 
                    className={cn(
                      "rounded-lg border p-2 text-xs",
                      entity.status === "approved" && "bg-green-50 dark:bg-green-950 border-green-200",
                      entity.status === "rejected" && "bg-red-50 dark:bg-red-950 border-red-200 opacity-50",
                      entity.status === "pending" && "bg-orange-50 dark:bg-orange-950 border-orange-200",
                      editingId === globalIdx && "ring-2 ring-blue-400"
                    )}
                  >
                    {/* Header */}
                    <div className="flex items-center justify-between mb-1">
                      <div className="flex items-center gap-1.5">
                        <span className={cn(
                          "px-1.5 py-0.5 rounded font-medium text-[10px]",
                          entity.entity_type === "MOMENT" && "bg-blue-100 text-blue-700",
                          entity.entity_type === "PULSE" && "bg-pink-100 text-pink-700",
                          entity.entity_type === "INTENT" && "bg-purple-100 text-purple-700",
                          entity.entity_type === "THREAD" && "bg-cyan-100 text-cyan-700",
                          entity.entity_type === "BOND" && "bg-amber-100 text-amber-700",
                          !["MOMENT", "PULSE", "INTENT", "THREAD", "BOND"].includes(entity.entity_type) && "bg-gray-100 text-gray-700"
                        )}>
                          {entity.entity_type}
                        </span>
                        <span className="text-muted-foreground text-[10px]">
                          {Math.round(entity.confidence * 100)}%
                        </span>
                        {entity.status === "approved" && (
                          <CheckCircle className="h-3 w-3 text-green-600" />
                        )}
                        {entity.status === "rejected" && (
                          <X className="h-3 w-3 text-red-600" />
                        )}
                      </div>
                      
                      {/* Action Buttons */}
                      {entity.status === "pending" && editingId !== globalIdx && (
                        <div className="flex items-center gap-0.5">
                          <Button 
                            variant="ghost" 
                            size="sm" 
                            className="h-5 w-5 p-0 text-green-600 hover:bg-green-100"
                            onClick={() => handleApprove(globalIdx)}
                            title="Approve"
                          >
                            <CheckCircle className="h-3 w-3" />
                          </Button>
                          <Button 
                            variant="ghost" 
                            size="sm" 
                            className="h-5 w-5 p-0 text-blue-600 hover:bg-blue-100"
                            onClick={() => handleEdit(globalIdx)}
                            title="Edit"
                          >
                            <Edit className="h-3 w-3" />
                          </Button>
                          <Button 
                            variant="ghost" 
                            size="sm" 
                            className="h-5 w-5 p-0 text-red-600 hover:bg-red-100"
                            onClick={() => handleReject(globalIdx)}
                            title="Reject"
                          >
                            <X className="h-3 w-3" />
                          </Button>
                        </div>
                      )}
                      
                      {entity.status === "approved" && editingId !== globalIdx && (
                        <Button 
                          variant="ghost" 
                          size="sm" 
                          className="h-5 w-5 p-0"
                          onClick={() => handleEdit(globalIdx)}
                          title="Edit"
                        >
                          <Edit className="h-3 w-3" />
                        </Button>
                      )}
                    </div>
                    
                    {/* Content */}
                    {editingId === globalIdx ? (
                      <div className="space-y-1.5">
                        <Textarea
                          value={editText}
                          onChange={(e) => setEditText(e.target.value)}
                          className="min-h-[50px] text-xs"
                          autoFocus
                        />
                        <div className="flex gap-1">
                          <Button size="sm" className="h-5 text-[10px]" onClick={() => handleSaveEdit(globalIdx)}>
                            Save
                          </Button>
                          <Button size="sm" variant="ghost" className="h-5 text-[10px]" onClick={handleCancelEdit}>
                            Cancel
                          </Button>
                        </div>
                      </div>
                    ) : (
                      <div className="text-muted-foreground text-[11px]">
                        {entity.edited_content || entity.content}
                      </div>
                    )}
                    
                    {/* Physics mini-display */}
                    {entity.physics && (
                      <div className="mt-1 flex gap-1.5 text-[9px] text-muted-foreground">
                        <span className={cn(
                          "px-1 rounded",
                          entity.physics.valence > 0.3 ? "bg-green-100" : 
                          entity.physics.valence < -0.3 ? "bg-red-100" : ""
                        )}>
                          V:{entity.physics.valence?.toFixed(1)}
                        </span>
                        <span>A:{entity.physics.arousal?.toFixed(1)}</span>
                        <span>S:{entity.physics.significance?.toFixed(1)}</span>
                      </div>
                    )}
                  </div>
                )
              })}
            </div>
          ))}
          
          {/* DB Status Notice */}
          <div className="text-[10px] text-muted-foreground bg-muted/50 rounded px-2 py-1 flex items-center gap-1">
            <Database className="h-3 w-3" />
            TigerData: <span className="text-orange-600">Not connected</span>
            <span className="text-muted-foreground">• Entities stored locally</span>
          </div>
        </div>
      )}
    </div>
  )
}

// ============================================================================
// Thread Item Display
// ============================================================================

function ThreadItemView({ 
  item, 
  isLast,
  onReply 
}: { 
  item: UIThreadItem
  isLast: boolean
  onReply: () => void 
}) {
  const isUser = item.role === "user"
  const isAssistant = item.role === "assistant"

  // User message in thread
  if (isUser) {
    return (
      <div className="ml-4 border-l-2 border-blue-200 pl-3 py-2">
        <div className="flex items-center gap-2 text-xs text-muted-foreground mb-1">
          <User className="h-3.5 w-3.5 text-blue-500" />
          <span className="font-medium text-foreground">You</span>
          <Clock className="h-2.5 w-2.5" />
          <span>{new Date(item.timestamp).toLocaleTimeString()}</span>
        </div>
        <div className="text-sm">{item.content}</div>
      </div>
    )
  }

  // Assistant response
  if (isAssistant) {
    const hasThinking = (item.thinking_steps?.length || 0) > 0 || (item.tool_calls?.length || 0) > 0
    
    return (
      <div className="ml-4 border-l-2 border-primary/20 pl-3 py-2">
        {/* Header */}
        <div className="flex items-center gap-2 text-xs text-muted-foreground mb-1">
          <Bot className="h-3.5 w-3.5 text-primary" />
          <span className="font-medium text-foreground">{item.agent_speaker || "Agent"}</span>
          <Clock className="h-2.5 w-2.5" />
          <span>{new Date(item.timestamp).toLocaleTimeString()}</span>
          {item.status && item.status !== "complete" && item.status !== "error" && (
            <span className="text-primary flex items-center gap-1 animate-pulse">
              <Sparkles className="h-2.5 w-2.5" />
              {item.status}
            </span>
          )}
        </div>

        {/* Live Chain of Thought - ALWAYS show when there's content */}
        {hasThinking && (
          <div className="mb-2">
            <ChainOfThoughtSection 
              thinkingSteps={item.thinking_steps} 
              toolCalls={item.tool_calls}
              defaultOpen={true}
              isLive={item.is_typing}
            />
          </div>
        )}

        {/* Content or Thinking indicator */}
        {item.is_typing ? (
          !hasThinking && <ThinkingIndicator activity={item.current_activity} />
        ) : (
          <div className="text-sm whitespace-pre-wrap prose prose-sm max-w-none dark:prose-invert">
            {item.content}
          </div>
        )}

        {/* Heddle Analysis + Weave Units + Spawned Entities (only after completion) */}
        {!item.is_typing && item.heddle_result && (
          <div className="mt-2 space-y-2">
            {/* Weave Units (segmented content) */}
            {(item.heddle_result as ExtendedHeddleResult).weave_units?.length && (
              <WeaveUnitsSection 
                weaveUnits={(item.heddle_result as ExtendedHeddleResult).weave_units!}
              />
            )}
            
            {/* Spawned Entities with HILT */}
            {(item.heddle_result as ExtendedHeddleResult).spawn?.suggestions?.length && (
              <SpawnedEntitiesSection 
                suggestions={(item.heddle_result as ExtendedHeddleResult).spawn!.suggestions!}
                content={item.content || ""}
              />
            )}
            
            {/* Summary Heddle Result (collapsed) */}
            <HeddleResultSection heddle={item.heddle_result as ExtendedHeddleResult} />
          </div>
        )}

        {/* Reply button on last complete response */}
        {isLast && !item.is_typing && item.status === "complete" && (
          <Button variant="ghost" size="sm" className="mt-2 h-6 text-xs" onClick={onReply}>
            <Reply className="h-3 w-3 mr-1" /> Reply
          </Button>
        )}
      </div>
    )
  }

  return null
}

// ============================================================================
// Summary Response (for inactive messages)
// ============================================================================

function SummaryResponse({ item, onClick }: { item: UIThreadItem; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className="ml-4 mt-1 flex items-start gap-2 p-2 rounded-lg bg-muted/50 hover:bg-muted transition-colors text-left w-full max-w-md"
    >
      <Bot className="h-4 w-4 text-primary mt-0.5 flex-shrink-0" />
      <div className="flex-1 min-w-0">
        <div className="text-xs text-muted-foreground mb-0.5">
          {item.agent_speaker || "Agent"} • {new Date(item.timestamp).toLocaleTimeString()}
        </div>
        <p className="text-sm text-muted-foreground truncate">
          {item.summary || item.content?.slice(0, 140) || "..."}
        </p>
      </div>
      <ChevronRight className="h-4 w-4 text-muted-foreground flex-shrink-0" />
    </button>
  )
}

// ============================================================================
// Message Block
// ============================================================================

function MessageBlock({ message }: { message: UIChannelMessage }) {
  const replyingToId = useChatStore((s) => s.replyingToMessageId)
  const setReplyingTo = useChatStore((s) => s.setReplyingTo)
  const toggleExpanded = useChatStore((s) => s.toggleMessageExpanded)
  const sendReply = useChatStore((s) => s.sendReply)
  const isLoading = useChatStore((s) => s.agenticLoading)
  
  const [replyText, setReplyText] = useState("")
  const isReplying = replyingToId === message.id
  
  // Get last assistant response for summary display
  const lastAssistantResponse = [...message.thread].reverse().find(t => t.role === "assistant")

  const handleSendReply = () => {
    if (replyText.trim()) {
      sendReply(message.id, replyText)
      setReplyText("")
    }
  }

  return (
    <div className={cn(
      "py-3 transition-colors",
      message.is_active && "bg-primary/5"
    )}>
      {/* Original user message */}
      <div className="flex items-start gap-2">
        <User className="h-5 w-5 text-muted-foreground mt-0.5 flex-shrink-0" />
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 text-xs text-muted-foreground mb-0.5">
            <span className="font-medium text-foreground">You</span>
            <span>{new Date(message.timestamp).toLocaleTimeString()}</span>
            {message.thread.length > 0 && !message.is_active && (
              <button 
                onClick={() => toggleExpanded(message.id)}
                className="flex items-center gap-1 text-primary hover:underline ml-2"
              >
                {message.is_expanded ? "Collapse" : `${message.thread.length} replies`}
                {message.is_expanded ? <ChevronDown className="h-3 w-3" /> : <ChevronRight className="h-3 w-3" />}
              </button>
            )}
          </div>
          <div className="text-sm">{message.content}</div>
        </div>
      </div>

      {/* Thread items */}
      {message.is_active || message.is_expanded ? (
        <div className="mt-2 space-y-2">
          {message.thread.map((item, i) => (
            <ThreadItemView
              key={item.id}
              item={item}
              isLast={i === message.thread.length - 1}
              onReply={() => setReplyingTo(message.id)}
            />
          ))}
        </div>
      ) : lastAssistantResponse ? (
        <SummaryResponse 
          item={lastAssistantResponse} 
          onClick={() => toggleExpanded(message.id)} 
        />
      ) : null}

      {/* Reply input */}
      {isReplying && (
        <div className="ml-4 border-l-2 border-primary/20 pl-3 mt-2">
          <div className="flex gap-2">
            <Textarea
              value={replyText}
              onChange={(e) => setReplyText(e.target.value)}
              placeholder="Reply..."
              className="min-h-[40px] text-sm resize-none"
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); handleSendReply() }
                if (e.key === "Escape") setReplyingTo(null)
              }}
              autoFocus
            />
            <div className="flex flex-col gap-1">
              <Button size="sm" className="h-7" onClick={handleSendReply} disabled={!replyText.trim() || isLoading}>
                <Send className="h-3 w-3" />
              </Button>
              <Button size="sm" variant="ghost" className="h-7" onClick={() => setReplyingTo(null)}>
                <X className="h-3 w-3" />
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

// ============================================================================
// Main Component
// ============================================================================

export function AgenticChat() {
  const channel = useChatStore(selectActiveChannel)
  const input = useChatStore((s) => s.agenticInput)
  const setInput = useChatStore((s) => s.setAgenticInput)
  const isLoading = useChatStore((s) => s.agenticLoading)
  const sendMessage = useChatStore((s) => s.sendMessage)
  const scrollRef = useRef<HTMLDivElement>(null)

  // Auto-scroll on new messages
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [channel?.messages.length])

  const handleSend = () => {
    if (input.trim() && !isLoading) {
      sendMessage(input)
    }
  }

  return (
    <div className="flex h-full">
      <ChannelList />
      
      <div className="flex-1 flex flex-col min-w-0">
        {/* Header */}
        <div className="p-2 border-b flex items-center gap-2">
          {channel && isFamilyChannel(channel) ? (
            <span className="text-base">👨‍👩‍👧‍👦</span>
          ) : (
            <Hash className="h-4 w-4 text-muted-foreground" />
          )}
          <span className="font-medium text-sm">{channel?.name || "Select channel"}</span>
          {channel && isFamilyChannel(channel) && (
            <span className="text-xs text-muted-foreground bg-muted px-1.5 py-0.5 rounded">
              Family
            </span>
          )}
        </div>

        {/* Messages */}
        <ScrollArea className="flex-1" ref={scrollRef}>
          <div className="p-3 divide-y">
            {channel?.messages.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12 text-center text-muted-foreground">
                <Bot className="h-10 w-10 mb-3 opacity-40" />
                <p className="text-sm">No messages yet</p>
                <p className="text-xs">Start a conversation below</p>
              </div>
            ) : (
              channel?.messages.map((m) => <MessageBlock key={m.id} message={m} />)
            )}
          </div>
        </ScrollArea>

        {/* Input */}
        <div className="p-2 border-t">
          {/* Family channel hint */}
          {channel && isFamilyChannel(channel) && (
            <div className="mb-2 px-2 py-1 bg-amber-50 dark:bg-amber-950 border border-amber-200 dark:border-amber-800 rounded text-xs text-amber-700 dark:text-amber-300 flex items-center gap-1.5">
              <span>👨‍👩‍👧‍👦</span>
              <span>Family channel — mention <code className="bg-amber-100 dark:bg-amber-900 px-1 rounded">@nona</code> to invoke AI</span>
              {hasAIMention(input) && (
                <span className="ml-auto text-green-600 dark:text-green-400 flex items-center gap-1">
                  <Bot className="h-3 w-3" /> AI will respond
                </span>
              )}
            </div>
          )}
          <div className="flex gap-2">
            <Textarea
              value={input}
              onChange={(e) => setInput(e.target.value)}
              placeholder={
                channel && isFamilyChannel(channel)
                  ? `Message #${channel.name}... (use @nona for AI)`
                  : `Message #${channel?.name || "channel"}...`
              }
              className="min-h-[40px] resize-none text-sm"
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); handleSend() }
              }}
            />
            <Button onClick={handleSend} disabled={!input.trim() || isLoading} className="self-end">
              {isLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <ArrowUp className="h-4 w-4" />}
            </Button>
          </div>
        </div>
      </div>
    </div>
  )
}

export default AgenticChat

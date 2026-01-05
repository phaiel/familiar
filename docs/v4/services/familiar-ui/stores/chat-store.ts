/**
 * Chat Store - Schema-First Implementation
 * 
 * Uses types generated from familiar-core Rust schemas.
 * Handles conversation context for proper threading.
 * Persists to TigerData API for conversation history.
 * 
 * Auth Integration:
 * - All API calls include Authorization header from auth-store
 * - Tenant ID comes from auth-store memberships via settings-store
 */
import { create } from "zustand"
import type { 
  UIChannel, 
  UIChannelMessage, 
  UIThreadItem, 
  UIToolCall, 
  UIThinkingStep,
  ConversationHistoryItem,
  Channel,
  Message,
  ChannelType,
} from "@/types"
import { useSettingsStore } from "./settings-store"
import { useAuthStore } from "./auth-store"

// ============================================================================
// API Types (aliases to generated types)
// ============================================================================

// ApiChannel = Channel from generated types
type ApiChannel = Channel

// ApiMessage = Message from generated types
type ApiMessage = Message

// ============================================================================
// Store Types (non-schema local state)
// ============================================================================

export type ChatTab = "heddle" | "agentic"

interface ChatStore {
  activeTab: ChatTab
  setActiveTab: (tab: ChatTab) => void

  // Initialization
  initialized: boolean
  loadChannels: () => Promise<void>

  // Heddle (simple flat chat)
  heddleMessages: Array<{ id: string; role: string; content: string; timestamp: string }>
  heddleInput: string
  heddleLoading: boolean
  setHeddleInput: (input: string) => void
  sendHeddleMessage: (text: string) => Promise<void>

  // Agentic (threaded)
  channels: UIChannel[]
  activeChannelId: string | null
  agenticInput: string
  agenticLoading: boolean
  replyingToMessageId: string | null
  
  setAgenticInput: (input: string) => void
  setActiveChannel: (id: string | null) => void
  setReplyingTo: (messageId: string | null) => void
  toggleMessageExpanded: (messageId: string) => void
  createChannel: (name: string) => Promise<string>
  sendMessage: (content: string) => Promise<void>
  sendReply: (messageId: string, content: string) => Promise<void>
}

// ============================================================================
// Helpers
// ============================================================================

function generateSummary(content: string): string {
  if (!content) return ""
  if (content.length <= 140) return content
  const plain = content.replace(/[#*`_\[\]()]/g, "").replace(/\n+/g, " ")
  return plain.slice(0, 137) + "..."
}

// AI mention patterns for family channel
const AI_MENTION_PATTERNS = [
  /@nona\b/i,
  /@ai\b/i,
  /@familiar\b/i,
  /@assistant\b/i,
]

/**
 * Check if a message mentions the AI (for family channel)
 */
function hasAIMention(content: string): boolean {
  return AI_MENTION_PATTERNS.some(pattern => pattern.test(content))
}

/**
 * Strip AI mention from message content
 */
function stripAIMention(content: string): string {
  let result = content
  for (const pattern of AI_MENTION_PATTERNS) {
    result = result.replace(pattern, "").trim()
  }
  return result
}

/**
 * Check if a channel is a family/shared channel (no owner = family)
 */
function isFamilyChannel(channel: UIChannel): boolean {
  // Family channels have specific naming convention or metadata
  // For now, check if name contains "family" or "shared"
  const name = channel.name.toLowerCase()
  return name.includes("family") || name.includes("shared")
}

function buildConversationHistory(message: UIChannelMessage): ConversationHistoryItem[] {
  const history: ConversationHistoryItem[] = []
  
  // Start with original message
  history.push({ role: "user", content: message.content })
  
  // Add all thread items (excluding typing indicators)
  for (const item of message.thread) {
    if (!item.is_typing && item.content) {
      history.push({ role: item.role, content: item.content })
    }
  }
  
  return history
}

function createTypingIndicator(id: string): UIThreadItem {
  return {
    id,
    role: "assistant",
    content: "",
    timestamp: new Date().toISOString(),
    agent_speaker: null,
    is_typing: true,
    status: "thinking",
    current_activity: "Processing...",
    thinking_steps: [],
    tool_calls: [],
    heddle_result: null,
    summary: null,
  }
}

// ============================================================================
// API Helpers
// ============================================================================

// Default tenant ID (will be replaced with real tenant from settings)
const DEFAULT_TENANT_ID = "00000000-0000-0000-0000-000000000001"

/**
 * Get authorization headers from auth store
 */
function getAuthHeaders(): HeadersInit {
  const { session } = useAuthStore.getState()
  const headers: HeadersInit = {
    "Content-Type": "application/json",
  }
  
  if (session?.token) {
    headers["Authorization"] = `Bearer ${session.token}`
  }
  
  return headers
}

/**
 * Get tenant ID from settings (synced from auth)
 */
function getTenantId(): string {
  const { tenantId } = useSettingsStore.getState()
  return tenantId || DEFAULT_TENANT_ID
}

// ============================================================================
// API Functions
// ============================================================================

async function fetchChannels(tenantId: string): Promise<ApiChannel[]> {
  try {
    const res = await fetch(`/api/tenants/${tenantId}/channels`, {
      headers: getAuthHeaders(),
    })
    if (!res.ok) return []
    const data = await res.json()
    return data.channels || []
  } catch {
    console.warn("Failed to fetch channels from API")
    return []
  }
}

async function createChannelApi(tenantId: string, name: string, channelType: "personal" | "family" = "personal"): Promise<ApiChannel | null> {
  try {
    const res = await fetch("/api/channels", {
      method: "POST",
      headers: getAuthHeaders(),
      body: JSON.stringify({
        tenant_id: tenantId,
        name,
        channel_type: channelType,
      }),
    })
    if (!res.ok) return null
    return res.json()
  } catch {
    console.warn("Failed to create channel via API")
    return null
  }
}

async function fetchMessages(channelId: string, limit = 50): Promise<ApiMessage[]> {
  try {
    const res = await fetch(`/api/channels/${channelId}/messages?limit=${limit}`, {
      headers: getAuthHeaders(),
    })
    if (!res.ok) return []
    const data = await res.json()
    return data.messages || []
  } catch {
    console.warn("Failed to fetch messages from API")
    return []
  }
}

async function saveMessageApi(channelId: string, role: string, content: string, agentSpeaker?: string, weaveResult?: unknown): Promise<ApiMessage | null> {
  try {
    const res = await fetch(`/api/channels/${channelId}/messages`, {
      method: "POST",
      headers: getAuthHeaders(),
      body: JSON.stringify({
        channel_id: channelId,
        role,
        content,
        agent_speaker: agentSpeaker,
        weave_result: weaveResult,
      }),
    })
    if (!res.ok) return null
    return res.json()
  } catch {
    console.warn("Failed to save message via API")
    return null
  }
}

async function callAgenticApi(
  content: string, 
  conversationHistory: ConversationHistoryItem[],
  courseId?: string,
  flowPath?: string
) {
  // Get flow path and tenant from settings store if not provided
  const settings = useSettingsStore.getState()
  const effectiveFlowPath = flowPath ?? settings.selectedFlowPath
  const tenantId = getTenantId()
  
  // #region agent log
  console.log("[callAgenticApi] Calling with flow_path:", effectiveFlowPath)
  // #endregion
  
  const res = await fetch("/api/agentic/command", {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify({
      command_type: "send_message",
      content,
      conversation_history: conversationHistory,
      course_id: courseId,
      tenant_id: tenantId,
      request_id: crypto.randomUUID(),
      flow_path: effectiveFlowPath,
    }),
  })
  const data = await res.json()
  // #region agent log
  console.log("[callAgenticApi] Response:", data)
  // #endregion
  return data
}

// Convert API channel to UI channel
function apiChannelToUI(channel: ApiChannel, messages: UIChannelMessage[] = []): UIChannel {
  return {
    id: channel.id,
    name: channel.name,
    messages,
    created_at: channel.created_at,
    updated_at: channel.updated_at,
  }
}

// Convert API messages to UI message format
function convertApiMessagesToUI(messages: ApiMessage[]): UIChannelMessage[] {
  // Group messages - for now treat each user message as a thread start
  // and assistant messages as thread items
  const result: UIChannelMessage[] = []
  let currentUserMessage: UIChannelMessage | null = null
  
  for (const msg of messages) {
    if (msg.role === "user") {
      // Save previous message if exists
      if (currentUserMessage) {
        result.push(currentUserMessage)
      }
      // Start new thread
      currentUserMessage = {
        id: msg.id,
        content: msg.content,
        timestamp: msg.created_at,
        thread: [],
        is_expanded: false,
        is_active: false,
      }
    } else if (msg.role === "assistant" && currentUserMessage) {
      // Add as thread item
      currentUserMessage.thread.push({
        id: msg.id,
        role: "assistant",
        content: msg.content,
        timestamp: msg.created_at,
        agent_speaker: msg.agent_speaker,
        is_typing: false,
        status: "complete",
        current_activity: null,
        thinking_steps: (msg.thinking_steps || []) as UIThinkingStep[],
        tool_calls: (msg.tool_calls || []) as UIToolCall[],
        heddle_result: msg.weave_result as any,
        summary: generateSummary(msg.content),
      })
    } else if (msg.role === "assistant") {
      // Orphan assistant message - create a synthetic user message
      result.push({
        id: `synthetic-${msg.id}`,
        content: "(Previous context)",
        timestamp: msg.created_at,
        thread: [{
          id: msg.id,
          role: "assistant",
          content: msg.content,
          timestamp: msg.created_at,
          agent_speaker: msg.agent_speaker,
          is_typing: false,
          status: "complete",
          current_activity: null,
          thinking_steps: [],
          tool_calls: [],
          heddle_result: null,
          summary: generateSummary(msg.content),
        }],
        is_expanded: false,
        is_active: false,
      })
    }
  }
  
  // Don't forget the last message
  if (currentUserMessage) {
    result.push(currentUserMessage)
  }
  
  return result
}

// ============================================================================
// Store
// ============================================================================

export const useChatStore = create<ChatStore>((set, get) => ({
  activeTab: "heddle",
  setActiveTab: (tab) => set({ activeTab: tab }),

  // Initialization
  initialized: false,
  loadChannels: async () => {
    const tenantId = getTenantId()
    
    try {
      const apiChannels = await fetchChannels(tenantId)
      
      if (apiChannels.length > 0) {
        // Load messages for each channel
        const uiChannels = await Promise.all(
          apiChannels.map(async (ch) => {
            const messages = await fetchMessages(ch.id)
            // Convert API messages to UI messages (grouped by parent thread)
            const uiMessages = convertApiMessagesToUI(messages)
            return apiChannelToUI(ch, uiMessages)
          })
        )
        
        // Find personal channel first, then family channel
        const personalChannel = uiChannels.find(c => 
          c.name.toLowerCase().includes('personal') || 
          apiChannels.find(ac => ac.id === c.id)?.channel_type === 'personal'
        )
        const defaultChannel = personalChannel || uiChannels[0]
        
        set({ 
          channels: uiChannels, 
          activeChannelId: defaultChannel?.id || null,
          initialized: true 
        })
      } else {
        // No channels from API, use default local channel
        set({ initialized: true })
      }
    } catch {
      // API unavailable, use local state
      set({ initialized: true })
    }
  },

  // Heddle
  heddleMessages: [],
  heddleInput: "",
  heddleLoading: false,
  setHeddleInput: (input) => set({ heddleInput: input }),
  sendHeddleMessage: async (text) => {
    if (!text.trim()) return
    const now = new Date().toISOString()
    set((s) => ({
      heddleMessages: [...s.heddleMessages, { id: crypto.randomUUID(), role: "user", content: text, timestamp: now }],
      heddleInput: "",
      heddleLoading: true,
    }))
    setTimeout(() => {
      set((s) => ({
        heddleMessages: [...s.heddleMessages, { id: crypto.randomUUID(), role: "assistant", content: `Echo: ${text}`, timestamp: new Date().toISOString() }],
        heddleLoading: false,
      }))
    }, 500)
  },

  // Agentic
  channels: [{
    id: "default",
    name: "General",
    messages: [],
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }],
  activeChannelId: "default",
  agenticInput: "",
  agenticLoading: false,
  replyingToMessageId: null,

  setAgenticInput: (input) => set({ agenticInput: input }),
  setActiveChannel: (id) => set({ activeChannelId: id, replyingToMessageId: null }),
  setReplyingTo: (messageId) => set({ replyingToMessageId: messageId }),
  
  toggleMessageExpanded: (messageId) => set((s) => ({
    channels: s.channels.map((ch) => ({
      ...ch,
      messages: ch.messages.map((m) => 
        m.id === messageId ? { ...m, is_expanded: !m.is_expanded } : m
      ),
    })),
  })),

  createChannel: async (name) => {
    const tenantId = getTenantId()
    
    // Try to create via API
    const apiChannel = await createChannelApi(tenantId, name)
    
    if (apiChannel) {
      const uiChannel = apiChannelToUI(apiChannel)
      set((s) => ({
        channels: [...s.channels, uiChannel],
        activeChannelId: uiChannel.id,
      }))
      return uiChannel.id
    }
    
    // Fallback to local creation
    const id = crypto.randomUUID()
    const now = new Date().toISOString()
    set((s) => ({
      channels: [...s.channels, { 
        id, 
        name, 
        messages: [], 
        created_at: now, 
        updated_at: now 
      }],
      activeChannelId: id,
    }))
    return id
  },

  sendMessage: async (content) => {
    const { activeChannelId, channels } = get()
    if (!content.trim() || !activeChannelId) return

    const activeChannel = channels.find(c => c.id === activeChannelId)
    const isFamily = activeChannel ? isFamilyChannel(activeChannel) : false
    const mentionsAI = hasAIMention(content)
    
    // In family channel, only invoke AI if mentioned
    const shouldInvokeAI = !isFamily || mentionsAI
    const contentForAI = mentionsAI ? stripAIMention(content) : content

    const messageId = crypto.randomUUID()
    const responseId = crypto.randomUUID()
    const now = new Date().toISOString()

    // Create new message (with or without typing indicator)
    const newMessage: UIChannelMessage = {
      id: messageId,
      content,
      timestamp: now,
      thread: shouldInvokeAI ? [createTypingIndicator(responseId)] : [],
      is_expanded: true,
      is_active: true,
    }

    set((s) => ({
      channels: s.channels.map((ch) =>
        ch.id === activeChannelId
          ? {
              ...ch,
              messages: [
                ...ch.messages.map((m) => ({ ...m, is_active: false })),
                newMessage,
              ],
              updated_at: now,
            }
          : ch
      ),
      agenticInput: "",
      agenticLoading: shouldInvokeAI,
    }))

    // Save user message to API
    saveMessageApi(activeChannelId, "user", content)

    // If in family channel without AI mention, just save the message (no AI call)
    if (!shouldInvokeAI) {
      return
    }

    // Build history (just this message for new thread)
    const history: ConversationHistoryItem[] = [{ role: "user", content: contentForAI }]

    try {
      const data = await callAgenticApi(contentForAI, history)
      
      if (data.job_id && data.ws_url) {
        // #region agent log
        console.log("[ChatStore] Connecting WebSocket:", `ws://localhost:3001${data.ws_url}`)
        // #endregion
        const socket = new WebSocket(`ws://localhost:3001${data.ws_url}`)
        
        // #region agent log
        socket.onopen = () => console.log("[ChatStore] WebSocket connected")
        socket.onerror = (e) => console.error("[ChatStore] WebSocket error:", e)
        socket.onclose = (e) => console.log("[ChatStore] WebSocket closed:", e.code, e.reason)
        // #endregion
        
        socket.onmessage = (event) => {
          // #region agent log
          console.log("[ChatStore] WebSocket message:", event.data)
          // #endregion
          const msg = JSON.parse(event.data)
          
          const updateResponse = (updates: Partial<UIThreadItem>) => {
            set((s) => ({
              channels: s.channels.map((ch) =>
                ch.id === activeChannelId
                  ? {
                      ...ch,
                      messages: ch.messages.map((m) =>
                        m.id === messageId
                          ? {
                              ...m,
                              thread: m.thread.map((t) =>
                                t.id === responseId ? { ...t, ...updates } : t
                              ),
                            }
                          : m
                      ),
                    }
                  : ch
              ),
            }))
          }

          if (msg.event_type === "thinking") {
            const current = get().channels.find(c => c.id === activeChannelId)
              ?.messages.find(m => m.id === messageId)
              ?.thread.find(t => t.id === responseId)
            
            const newStep: UIThinkingStep = {
              id: msg.id || crypto.randomUUID(),
              agent: msg.agent || "concierge",
              thought: msg.thought,
              timestamp: msg.timestamp || new Date().toISOString(),
            }
            
            // Avoid duplicates
            const existingSteps = current?.thinking_steps || []
            const alreadyExists = existingSteps.some(s => s.thought === msg.thought)
            
            if (!alreadyExists) {
              updateResponse({
                status: "thinking",
                current_activity: msg.thought,
                thinking_steps: [...existingSteps, newStep],
              })
            }
          } else if (msg.event_type === "tool_call") {
            const current = get().channels.find(c => c.id === activeChannelId)
              ?.messages.find(m => m.id === messageId)
              ?.thread.find(t => t.id === responseId)
            
            const newCall: UIToolCall = {
              id: crypto.randomUUID(),
              tool: msg.tool,
              arguments: msg.arguments || null,
              result: null,
              status: "running",
            }
            
            updateResponse({
              status: "calling_tools",
              current_activity: `Calling ${msg.tool}...`,
              tool_calls: [...(current?.tool_calls || []), newCall],
            })
          } else if (msg.event_type === "tool_result") {
            const current = get().channels.find(c => c.id === activeChannelId)
              ?.messages.find(m => m.id === messageId)
              ?.thread.find(t => t.id === responseId)
            
            updateResponse({
              tool_calls: (current?.tool_calls || []).map((t) =>
                t.tool === msg.tool && t.status === "running"
                  ? { ...t, result: msg.result, status: "complete" }
                  : t
              ) as UIToolCall[],
            })
          } else if (msg.event_type === "message_received" || msg.event_type === "task_completed") {
            const result = msg.result || msg
            const finalContent = result.response || msg.content || ""
            
            // Get current state to merge with
            const current = get().channels.find(c => c.id === activeChannelId)
              ?.messages.find(m => m.id === messageId)
              ?.thread.find(t => t.id === responseId)
            
            // Process thinking steps (use result's steps, or keep existing if none)
            const resultSteps = result.thinking_steps || []
            const thinkingSteps: UIThinkingStep[] = resultSteps.length > 0 
              ? resultSteps.map((s: any) => ({
                  id: s.id || crypto.randomUUID(),
                  agent: s.agent || "concierge",
                  thought: s.thought,
                  timestamp: s.timestamp || new Date().toISOString(),
                }))
              : (current?.thinking_steps || [])
            
            // Process tool calls (use result's calls, or keep existing if none)
            const resultTools = result.tool_calls || []
            const toolCalls: UIToolCall[] = resultTools.length > 0
              ? resultTools.map((t: any) => ({
                  id: t.id || crypto.randomUUID(),
                  tool: t.tool,
                  arguments: t.arguments || null,
                  result: t.result || null,
                  status: t.status || "complete" as const,
                }))
              : (current?.tool_calls || [])
            
            // Support both weave_result (new Fates) and heddle_result (legacy)
            const weaveOrHeddleResult = result.weave_result || result.heddle_result || null
            
            updateResponse({
              content: finalContent,
              summary: generateSummary(finalContent),
              agent_speaker: result.agent || "nona",
              is_typing: false,
              status: "complete",
              current_activity: null,
              thinking_steps: thinkingSteps,
              tool_calls: toolCalls,
              heddle_result: weaveOrHeddleResult,
            })
            
            // Save assistant message to API
            saveMessageApi(activeChannelId, "assistant", finalContent, result.agent || "nona", weaveOrHeddleResult)
            
            set({ agenticLoading: false })
            socket.close()
          } else if (msg.event_type === "error") {
            updateResponse({
              content: `Error: ${msg.error || msg.message}`,
              is_typing: false,
              status: "error",
            })
            set({ agenticLoading: false })
            socket.close()
          }
        }

        socket.onerror = () => {
          set((s) => ({
            channels: s.channels.map((ch) =>
              ch.id === activeChannelId
                ? {
                    ...ch,
                    messages: ch.messages.map((m) =>
                      m.id === messageId
                        ? {
                            ...m,
                            thread: m.thread.map((t) =>
                              t.id === responseId
                                ? { ...t, content: "Connection error", is_typing: false, status: "error" }
                                : t
                            ),
                          }
                        : m
                    ),
                  }
                : ch
            ),
            agenticLoading: false,
          }))
        }
      }
    } catch (e) {
      set((s) => ({
        channels: s.channels.map((ch) =>
          ch.id === activeChannelId
            ? {
                ...ch,
                messages: ch.messages.map((m) =>
                  m.id === messageId
                    ? {
                        ...m,
                        thread: m.thread.map((t) =>
                          t.id === responseId
                            ? { ...t, content: `Error: ${e}`, is_typing: false, status: "error" }
                            : t
                        ),
                      }
                    : m
                ),
              }
            : ch
        ),
        agenticLoading: false,
      }))
    }
  },

  sendReply: async (messageId, content) => {
    const { activeChannelId, channels } = get()
    if (!content.trim() || !activeChannelId) return

    const channel = channels.find(c => c.id === activeChannelId)
    const message = channel?.messages.find(m => m.id === messageId)
    if (!message) return

    const userReplyId = crypto.randomUUID()
    const aiResponseId = crypto.randomUUID()
    const now = new Date().toISOString()

    // User's reply item
    const userReply: UIThreadItem = {
      id: userReplyId,
      role: "user",
      content,
      timestamp: now,
      agent_speaker: null,
      is_typing: false,
      status: null,
      current_activity: null,
      thinking_steps: [],
      tool_calls: [],
      heddle_result: null,
      summary: null,
    }

    // Add user reply + typing indicator
    set((s) => ({
      channels: s.channels.map((ch) =>
        ch.id === activeChannelId
          ? {
              ...ch,
              messages: ch.messages.map((m) =>
                m.id === messageId
                  ? {
                      ...m,
                      is_active: true,
                      is_expanded: true,
                      thread: [...m.thread, userReply, createTypingIndicator(aiResponseId)],
                    }
                  : { ...m, is_active: false }
              ),
            }
          : ch
      ),
      replyingToMessageId: null,
      agenticLoading: true,
    }))

    // Build full conversation history including new reply
    const updatedChannel = get().channels.find(c => c.id === activeChannelId)
    const updatedMessage = updatedChannel?.messages.find(m => m.id === messageId)
    const history = updatedMessage ? buildConversationHistory(updatedMessage) : [{ role: "user", content }]

    try {
      const data = await callAgenticApi(content, history, messageId)
      
      if (data.job_id && data.ws_url) {
        const socket = new WebSocket(`ws://localhost:3001${data.ws_url}`)
        
        socket.onmessage = (event) => {
          const msg = JSON.parse(event.data)
          
          const updateResponse = (updates: Partial<UIThreadItem>) => {
            set((s) => ({
              channels: s.channels.map((ch) =>
                ch.id === activeChannelId
                  ? {
                      ...ch,
                      messages: ch.messages.map((m) =>
                        m.id === messageId
                          ? {
                              ...m,
                              thread: m.thread.map((t) =>
                                t.id === aiResponseId ? { ...t, ...updates } : t
                              ),
                            }
                          : m
                      ),
                    }
                  : ch
              ),
            }))
          }

          if (msg.event_type === "thinking") {
            const current = get().channels.find(c => c.id === activeChannelId)
              ?.messages.find(m => m.id === messageId)
              ?.thread.find(t => t.id === aiResponseId)
            
            const newStep: UIThinkingStep = {
              id: msg.id || crypto.randomUUID(),
              agent: msg.agent || "concierge",
              thought: msg.thought,
              timestamp: msg.timestamp || new Date().toISOString(),
            }
            
            const existingSteps = current?.thinking_steps || []
            const alreadyExists = existingSteps.some(s => s.thought === msg.thought)
            
            if (!alreadyExists) {
              updateResponse({
                status: "thinking",
                current_activity: msg.thought,
                thinking_steps: [...existingSteps, newStep],
              })
            }
          } else if (msg.event_type === "message_received" || msg.event_type === "task_completed") {
            const result = msg.result || msg
            const finalContent = result.response || msg.content || ""
            
            // Get current state to merge with
            const current = get().channels.find(c => c.id === activeChannelId)
              ?.messages.find(m => m.id === messageId)
              ?.thread.find(t => t.id === aiResponseId)
            
            // Process thinking steps
            const resultSteps = result.thinking_steps || []
            const thinkingSteps: UIThinkingStep[] = resultSteps.length > 0 
              ? resultSteps.map((s: any) => ({
                  id: s.id || crypto.randomUUID(),
                  agent: s.agent || "concierge",
                  thought: s.thought,
                  timestamp: s.timestamp || new Date().toISOString(),
                }))
              : (current?.thinking_steps || [])
            
            // Process tool calls
            const resultTools = result.tool_calls || []
            const toolCalls: UIToolCall[] = resultTools.length > 0
              ? resultTools.map((t: any) => ({
                  id: t.id || crypto.randomUUID(),
                  tool: t.tool,
                  arguments: t.arguments || null,
                  result: t.result || null,
                  status: t.status || "complete",
                }))
              : (current?.tool_calls || [])
            
            // Support both weave_result (new Fates) and heddle_result (legacy)
            const weaveOrHeddleResult = result.weave_result || result.heddle_result || null
            
            updateResponse({
              content: finalContent,
              summary: generateSummary(finalContent),
              agent_speaker: result.agent || "nona",
              is_typing: false,
              status: "complete",
              current_activity: null,
              thinking_steps: thinkingSteps,
              tool_calls: toolCalls,
              heddle_result: weaveOrHeddleResult,
            })
            
            // Save assistant message to API
            const channel = get().channels.find(c => c.id === activeChannelId)
            if (channel) {
              saveMessageApi(channel.id, "assistant", finalContent, result.agent || "nona", weaveOrHeddleResult)
            }
            
            set({ agenticLoading: false })
            socket.close()
          } else if (msg.event_type === "error") {
            updateResponse({
              content: `Error: ${msg.error}`,
              is_typing: false,
              status: "error",
            })
            set({ agenticLoading: false })
            socket.close()
          }
        }

        socket.onerror = () => {
          set((s) => ({
            channels: s.channels.map((ch) =>
              ch.id === activeChannelId
                ? {
                    ...ch,
                    messages: ch.messages.map((m) =>
                      m.id === messageId
                        ? {
                            ...m,
                            thread: m.thread.map((t) =>
                              t.id === aiResponseId
                                ? { ...t, content: "Connection error", is_typing: false, status: "error" }
                                : t
                            ),
                          }
                        : m
                    ),
                  }
                : ch
            ),
            agenticLoading: false,
          }))
        }
      }
    } catch {
      set((s) => ({
        channels: s.channels.map((ch) =>
          ch.id === activeChannelId
            ? {
                ...ch,
                messages: ch.messages.map((m) =>
                  m.id === messageId
                    ? {
                        ...m,
                        thread: m.thread.map((t) =>
                          t.id === aiResponseId
                            ? { ...t, content: "Error occurred", is_typing: false, status: "error" }
                            : t
                        ),
                      }
                    : m
                ),
              }
            : ch
        ),
        agenticLoading: false,
      }))
    }
  },
}))

export const selectActiveChannel = (s: ChatStore) => s.channels.find((c) => c.id === s.activeChannelId)

// Export helpers for family channel detection
export { hasAIMention, isFamilyChannel }

// Re-export types for convenience
export type { UIChannel, UIChannelMessage, UIThreadItem, UIToolCall, UIThinkingStep }

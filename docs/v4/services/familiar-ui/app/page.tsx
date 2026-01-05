"use client"

import { useEffect } from "react"
import { ChatInterface } from "@/components/chat/chat-interface"
import { AgenticChat } from "@/components/chat/agentic-chat"
import { NavSidebar } from "@/components/sidebar/nav-sidebar"
import { useChatStore } from "@/stores/chat-store"
import { useAuthStore } from "@/stores/auth-store"
import { useSettingsStore } from "@/stores/settings-store"

export default function Home() {
  const activeTab = useChatStore((state) => state.activeTab)
  const loadChannels = useChatStore((state) => state.loadChannels)
  
  const { user, isAuthenticated, memberships } = useAuthStore()
  const { tenantId } = useSettingsStore()

  // Load channels when tenant is available
  useEffect(() => {
    if (isAuthenticated && tenantId) {
      loadChannels()
    }
  }, [isAuthenticated, tenantId, loadChannels])

  // Show user context in the UI
  const primaryMembership = memberships.find(m => m.is_primary) || memberships[0]

  return (
    <main className="min-h-screen bg-background flex">
      {/* Navigation Sidebar */}
      <NavSidebar />
      
      {/* Main Content */}
      <div className="flex-1 flex flex-col h-screen overflow-hidden">
        {/* Tab Content */}
        {activeTab === "heddle" ? (
          <ChatInterface />
        ) : (
          <AgenticChat />
        )}
      </div>
    </main>
  )
}

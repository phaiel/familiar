"use client"

import React from "react"
import { MessageSquare, Bot, Settings, HelpCircle, Check, Users, User, LogOut, Home, Hash } from "lucide-react"
import { cn } from "@/lib/utils"
import { Button } from "@/components/ui/button"
import { Tooltip, TooltipContent, TooltipTrigger, TooltipProvider } from "@/components/ui/tooltip"
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover"
import { Separator } from "@/components/ui/separator"
import { useChatStore, type ChatTab } from "@/stores/chat-store"
import { useSettingsStore, AVAILABLE_FLOWS } from "@/stores/settings-store"
import { useAuthStore } from "@/stores/auth-store"

interface NavItemProps {
  icon: React.ElementType
  label: string
  tab?: ChatTab
  isActive?: boolean
  onClick?: () => void
  disabled?: boolean
}

function NavItem({ icon: Icon, label, tab, isActive, onClick, disabled }: NavItemProps) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          onClick={onClick}
          disabled={disabled}
          className={cn(
            "h-10 w-10 rounded-lg transition-colors",
            isActive
              ? "bg-primary text-primary-foreground"
              : "text-muted-foreground hover:text-foreground hover:bg-muted"
          )}
        >
          <Icon className="h-5 w-5" />
          <span className="sr-only">{label}</span>
        </Button>
      </TooltipTrigger>
      <TooltipContent side="right" sideOffset={8}>
        {label}
      </TooltipContent>
    </Tooltip>
  )
}

function SettingsPopover() {
  const isOpen = useSettingsStore((s) => s.isSettingsOpen)
  const setOpen = useSettingsStore((s) => s.setSettingsOpen)
  const selectedFlowId = useSettingsStore((s) => s.selectedFlowId)
  const setSelectedFlow = useSettingsStore((s) => s.setSelectedFlow)
  const tenantName = useSettingsStore((s) => s.tenantName)
  const currentUserName = useSettingsStore((s) => s.currentUserName)
  const logout = useAuthStore((s) => s.logout)

  const handleLogout = async () => {
    await logout()
    setOpen(false)
  }

  return (
    <Popover open={isOpen} onOpenChange={setOpen}>
      <Tooltip>
        <TooltipTrigger asChild>
          <PopoverTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className={cn(
                "h-10 w-10 rounded-lg transition-colors",
                isOpen
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:text-foreground hover:bg-muted"
              )}
            >
              <Settings className="h-5 w-5" />
              <span className="sr-only">Settings</span>
            </Button>
          </PopoverTrigger>
        </TooltipTrigger>
        <TooltipContent side="right" sideOffset={8}>
          Settings
        </TooltipContent>
      </Tooltip>
      <PopoverContent side="right" align="end" className="w-80 p-3">
        <div className="space-y-4">
          {/* Tenant Info */}
          <div className="space-y-2">
            <div className="font-medium text-sm flex items-center gap-2">
              <Users className="h-4 w-4" />
              Family
            </div>
            <div className="flex items-center gap-2 p-2 bg-muted/50 rounded-md">
              <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center">
                <span className="text-lg">👨‍👩‍👧‍👦</span>
              </div>
              <div className="flex-1 min-w-0">
                <div className="font-medium text-sm">{tenantName}</div>
                {currentUserName && (
                  <div className="text-xs text-muted-foreground flex items-center gap-1">
                    <User className="h-3 w-3" />
                    {currentUserName}
                  </div>
                )}
              </div>
            </div>
          </div>

          <Separator />

          {/* Flow Selection */}
          <div className="space-y-2">
            <div className="font-medium text-sm">Agentic Flow</div>
            <p className="text-xs text-muted-foreground">
              Select which Windmill flow to use for agent processing
            </p>
            <div className="space-y-1">
              {AVAILABLE_FLOWS.map((flow) => (
                <button
                  key={flow.id}
                  onClick={() => setSelectedFlow(flow.id)}
                  className={cn(
                    "w-full flex items-start gap-2 p-2 rounded-md text-left transition-colors",
                    "hover:bg-muted",
                    selectedFlowId === flow.id && "bg-muted"
                  )}
                >
                  <div className={cn(
                    "w-4 h-4 rounded-full border flex items-center justify-center flex-shrink-0 mt-0.5",
                    selectedFlowId === flow.id 
                      ? "border-primary bg-primary text-primary-foreground" 
                      : "border-muted-foreground"
                  )}>
                    {selectedFlowId === flow.id && <Check className="h-2.5 w-2.5" />}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="font-medium text-sm">{flow.name}</div>
                    <div className="text-xs text-muted-foreground truncate">{flow.path}</div>
                    <div className="text-xs text-muted-foreground mt-0.5">{flow.description}</div>
                  </div>
                </button>
              ))}
            </div>
          </div>

          <Separator />

          {/* Footer */}
          <div className="text-[10px] text-muted-foreground">
            Flow: <code className="bg-muted px-1 rounded">{
              AVAILABLE_FLOWS.find(f => f.id === selectedFlowId)?.path
            }</code>
          </div>

          <Separator />

          {/* Logout */}
          <Button 
            variant="ghost" 
            className="w-full justify-start text-destructive hover:text-destructive hover:bg-destructive/10"
            onClick={handleLogout}
          >
            <LogOut className="h-4 w-4 mr-2" />
            Sign Out
          </Button>
        </div>
      </PopoverContent>
    </Popover>
  )
}

export function NavSidebar() {
  const activeTab = useChatStore((state) => state.activeTab)
  const setActiveTab = useChatStore((state) => state.setActiveTab)

  return (
    <TooltipProvider delayDuration={100}>
      <nav className="w-16 border-r bg-background flex flex-col items-center py-4 gap-2">
        {/* Logo/Brand */}
        <div className="mb-4">
          <span className="text-2xl">🧵</span>
        </div>

        {/* Main Navigation */}
        <div className="flex flex-col gap-2">
          <NavItem 
            icon={MessageSquare} 
            label="Heddle Chat" 
            isActive={activeTab === "heddle"}
            onClick={() => setActiveTab("heddle")}
          />
          <NavItem 
            icon={Bot} 
            label="Agent Chat" 
            isActive={activeTab === "agentic"}
            onClick={() => setActiveTab("agentic")}
          />
        </div>

        {/* Spacer */}
        <div className="flex-1" />

        {/* Bottom Navigation */}
        <div className="flex flex-col gap-2">
          <NavItem icon={HelpCircle} label="Help" disabled />
          <SettingsPopover />
        </div>
      </nav>
    </TooltipProvider>
  )
}

export default NavSidebar

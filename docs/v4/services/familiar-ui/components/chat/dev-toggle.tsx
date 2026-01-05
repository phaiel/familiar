"use client"

import React from "react"
import { Switch } from "@/components/ui/switch"
import { Label } from "@/components/ui/label"
import { Code2 } from "lucide-react"

interface DevToggleProps {
  enabled: boolean
  onToggle: (enabled: boolean) => void
}

export function DevToggle({ enabled, onToggle }: DevToggleProps) {
  return (
    <div className="flex items-center gap-2">
      <Switch id="dev-mode" checked={enabled} onCheckedChange={onToggle} />
      <Label htmlFor="dev-mode" className="flex items-center gap-1.5 text-xs text-muted-foreground cursor-pointer select-none">
        <Code2 className="w-3.5 h-3.5" />
        Dev Mode
      </Label>
    </div>
  )
}






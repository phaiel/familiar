"use client"

import React from "react"

export default function OnboardingLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-900 dark:to-slate-800 flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        {/* Logo/Branding */}
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-primary/10 mb-4">
            <span className="text-4xl">🧵</span>
          </div>
          <h1 className="text-2xl font-bold text-foreground">Familiar</h1>
          <p className="text-muted-foreground text-sm mt-1">Your family&apos;s AI companion</p>
        </div>
        
        {/* Content */}
        {children}
      </div>
    </div>
  )
}


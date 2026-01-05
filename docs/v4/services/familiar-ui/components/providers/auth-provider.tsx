"use client"

import React, { useEffect, useState } from "react"
import { useRouter, usePathname } from "next/navigation"
import { useAuthStore } from "@/stores/auth-store"
import { useSettingsStore } from "@/stores/settings-store"
import { Loader2 } from "lucide-react"

interface AuthProviderProps {
  children: React.ReactNode
}

/**
 * AuthProvider - Handles authentication state and route protection
 * 
 * Responsibilities:
 * - Initialize auth state from persisted session on mount
 * - Redirect unauthenticated users to onboarding
 * - Sync user/tenant info with settings store after auth
 * - Show loading state while checking authentication
 */
export function AuthProvider({ children }: AuthProviderProps) {
  const router = useRouter()
  const pathname = usePathname()
  
  const { 
    isAuthenticated, 
    user,
    memberships,
    needsFamily,
    initialize,
    isLoading: authLoading,
  } = useAuthStore()
  
  const { setTenant, setCurrentUser } = useSettingsStore()
  
  const [isInitialized, setIsInitialized] = useState(false)

  // Initialize auth state on mount
  useEffect(() => {
    const init = async () => {
      await initialize()
      setIsInitialized(true)
    }
    init()
  }, [initialize])

  // Sync auth state with settings store
  useEffect(() => {
    if (isAuthenticated && user) {
      // Set current user in settings
      setCurrentUser(user.id, user.name)
      
      // Set primary tenant if user has memberships
      const primaryMembership = memberships.find(m => m.is_primary) || memberships[0]
      if (primaryMembership) {
        setTenant(primaryMembership.tenant_id, primaryMembership.tenant_name)
      }
    }
  }, [isAuthenticated, user, memberships, setCurrentUser, setTenant])

  // Handle route protection
  useEffect(() => {
    if (!isInitialized) return
    
    const isOnboardingRoute = pathname.startsWith("/onboarding")
    
    if (!isAuthenticated && !isOnboardingRoute) {
      // Not logged in, redirect to onboarding
      router.push("/onboarding")
    } else if (isAuthenticated && needsFamily && !pathname.startsWith("/onboarding/create-family")) {
      // Logged in but needs to create/join a family
      router.push("/onboarding/create-family")
    } else if (isAuthenticated && !needsFamily && isOnboardingRoute && pathname !== "/onboarding/create-family") {
      // Fully authenticated, redirect away from onboarding
      router.push("/")
    }
  }, [isAuthenticated, needsFamily, pathname, router, isInitialized])

  // Show loading state while initializing
  if (!isInitialized || authLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="flex flex-col items-center gap-4">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          <p className="text-sm text-muted-foreground">Loading...</p>
        </div>
      </div>
    )
  }

  return <>{children}</>
}


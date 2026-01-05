/**
 * Auth Store - Authentication State Management
 * 
 * Manages user authentication state including:
 * - Current user session
 * - Login/signup flows
 * - Magic link handling
 * - Family memberships
 */
import { create } from "zustand"
import { persist } from "zustand/middleware"
import type {
  User,
  UserMembership,
  Session,
  AuthResponse,
  CurrentUserResponse,
  InvitationInfo,
} from "@/types"

// Re-export types for backward compatibility with imports from auth-store
export type { User, UserMembership, Session, AuthResponse, CurrentUserResponse, InvitationInfo }

// ============================================================================
// API Functions
// ============================================================================

const API_BASE = "/api"

async function apiSignup(email: string, password: string, name: string, inviteCode?: string): Promise<AuthResponse> {
  // #region agent log
  const signupUrl = `${API_BASE}/auth/signup`;
  fetch('http://127.0.0.1:7242/ingest/3bf1328a-5a4c-4ec6-a5bb-3cc02a78c23f',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({location:'auth-store.ts:31',message:'apiSignup called',data:{url:signupUrl,email,hasPassword:!!password,hasName:!!name},timestamp:Date.now(),sessionId:'debug-session',runId:'run1',hypothesisId:'A'})}).catch(()=>{});
  // #endregion
  
  // #region agent log
  fetch('http://127.0.0.1:7242/ingest/3bf1328a-5a4c-4ec6-a5bb-3cc02a78c23f',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({location:'auth-store.ts:32',message:'Before fetch',data:{fullUrl:window.location.origin+signupUrl,apiBase:API_BASE},timestamp:Date.now(),sessionId:'debug-session',runId:'run1',hypothesisId:'B'})}).catch(()=>{});
  // #endregion
  
  const res = await fetch(`${API_BASE}/auth/signup`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      email,
      password,
      name,
      invite_code: inviteCode,
      accept_terms: true,
      accept_privacy: true,
    }),
  }).catch((err) => {
    // #region agent log
    fetch('http://127.0.0.1:7242/ingest/3bf1328a-5a4c-4ec6-a5bb-3cc02a78c23f',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({location:'auth-store.ts:45',message:'Fetch error caught',data:{error:err.message,errorName:err.name,url:signupUrl},timestamp:Date.now(),sessionId:'debug-session',runId:'run1',hypothesisId:'C'})}).catch(()=>{});
    // #endregion
    throw err;
  });
  
  // #region agent log
  fetch('http://127.0.0.1:7242/ingest/3bf1328a-5a4c-4ec6-a5bb-3cc02a78c23f',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({location:'auth-store.ts:47',message:'Fetch response received',data:{status:res.status,statusText:res.statusText,ok:res.ok,url:res.url},timestamp:Date.now(),sessionId:'debug-session',runId:'run1',hypothesisId:'D'})}).catch(()=>{});
  // #endregion
  
  if (!res.ok) {
    const error = await res.json()
    // #region agent log
    fetch('http://127.0.0.1:7242/ingest/3bf1328a-5a4c-4ec6-a5bb-3cc02a78c23f',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({location:'auth-store.ts:50',message:'Response not ok',data:{status:res.status,error},timestamp:Date.now(),sessionId:'debug-session',runId:'run1',hypothesisId:'E'})}).catch(()=>{});
    // #endregion
    throw new Error(error.error || "Signup failed")
  }
  
  return res.json()
}

async function apiLogin(email: string, password: string): Promise<AuthResponse> {
  const res = await fetch(`${API_BASE}/auth/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, password }),
  })
  
  if (!res.ok) {
    const error = await res.json()
    throw new Error(error.error || "Login failed")
  }
  
  return res.json()
}

async function apiRequestMagicLink(email: string, inviteCode?: string): Promise<{ success: boolean; dev_token?: string }> {
  const res = await fetch(`${API_BASE}/auth/magic-link`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, invite_code: inviteCode }),
  })
  
  if (!res.ok) {
    const error = await res.json()
    throw new Error(error.error || "Failed to send magic link")
  }
  
  return res.json()
}

async function apiConsumeMagicLink(token: string): Promise<AuthResponse> {
  const res = await fetch(`${API_BASE}/auth/magic-link/${token}`)
  
  if (!res.ok) {
    const error = await res.json()
    throw new Error(error.error || "Invalid magic link")
  }
  
  return res.json()
}

async function apiLogout(token: string): Promise<void> {
  await fetch(`${API_BASE}/auth/logout`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Authorization": `Bearer ${token}`,
    },
  })
}

async function apiGetCurrentUser(token: string): Promise<CurrentUserResponse> {
  const res = await fetch(`${API_BASE}/auth/me`, {
    headers: { "Authorization": `Bearer ${token}` },
  })
  
  if (!res.ok) {
    throw new Error("Not authenticated")
  }
  
  return res.json()
}

async function apiGetInvitationByCode(code: string): Promise<InvitationInfo> {
  const res = await fetch(`${API_BASE}/invitations/code/${code}`)
  
  if (!res.ok) {
    const error = await res.json()
    throw new Error(error.error || "Invalid invitation code")
  }
  
  return res.json()
}

async function apiCreateFamily(token: string, name: string): Promise<{ id: string; name: string }> {
  console.log("[API] Creating family:", name)
  
  // Create tenant
  const res = await fetch(`${API_BASE}/tenants`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Authorization": `Bearer ${token}`,
    },
    body: JSON.stringify({ name }),
  })
  
  console.log("[API] Response status:", res.status)
  
  if (!res.ok) {
    const error = await res.json()
    console.error("[API] Error response:", error)
    throw new Error(error.error || "Failed to create family")
  }
  
  const result = await res.json()
  console.log("[API] Success response:", result)
  return result
}

// ============================================================================
// Store
// ============================================================================

interface AuthStore {
  // State
  user: User | null
  session: Session | null
  memberships: UserMembership[]
  isAuthenticated: boolean
  isLoading: boolean
  error: string | null
  needsFamily: boolean
  
  // Invitation state (for onboarding flow)
  pendingInvitation: InvitationInfo | null
  
  // Actions
  signup: (email: string, password: string, name: string, inviteCode?: string) => Promise<void>
  login: (email: string, password: string) => Promise<void>
  requestMagicLink: (email: string, inviteCode?: string) => Promise<{ dev_token?: string }>
  consumeMagicLink: (token: string) => Promise<void>
  logout: () => Promise<void>
  refreshUser: () => Promise<void>
  checkInviteCode: (code: string) => Promise<InvitationInfo>
  setPendingInvitation: (invitation: InvitationInfo | null) => void
  createFamily: (name: string) => Promise<{ id: string; name: string }>
  clearError: () => void
  
  // Initialization
  initialize: () => Promise<void>
}

export const useAuthStore = create<AuthStore>()(
  persist(
    (set, get) => ({
      // Initial state
      user: null,
      session: null,
      memberships: [],
      isAuthenticated: false,
      isLoading: false,
      error: null,
      needsFamily: false,
      pendingInvitation: null,

      // Signup with email + password
      signup: async (email, password, name, inviteCode) => {
        set({ isLoading: true, error: null })
        
        try {
          const response = await apiSignup(email, password, name, inviteCode)
          
          set({
            user: response.user,
            session: response.session,
            isAuthenticated: true,
            needsFamily: response.needs_family,
            isLoading: false,
          })
          
          // Refresh memberships
          await get().refreshUser()
        } catch (e) {
          set({
            error: e instanceof Error ? e.message : "Signup failed",
            isLoading: false,
          })
          throw e
        }
      },

      // Login with email + password
      login: async (email, password) => {
        set({ isLoading: true, error: null })
        
        try {
          const response = await apiLogin(email, password)
          
          set({
            user: response.user,
            session: response.session,
            isAuthenticated: true,
            needsFamily: response.needs_family,
            isLoading: false,
          })
          
          // Refresh memberships
          await get().refreshUser()
        } catch (e) {
          set({
            error: e instanceof Error ? e.message : "Login failed",
            isLoading: false,
          })
          throw e
        }
      },

      // Request magic link
      requestMagicLink: async (email, inviteCode) => {
        set({ isLoading: true, error: null })
        
        try {
          const result = await apiRequestMagicLink(email, inviteCode)
          set({ isLoading: false })
          return result
        } catch (e) {
          set({
            error: e instanceof Error ? e.message : "Failed to send magic link",
            isLoading: false,
          })
          throw e
        }
      },

      // Consume magic link
      consumeMagicLink: async (token) => {
        set({ isLoading: true, error: null })
        
        try {
          const response = await apiConsumeMagicLink(token)
          
          set({
            user: response.user,
            session: response.session,
            isAuthenticated: true,
            needsFamily: response.needs_family,
            isLoading: false,
          })
          
          // Refresh memberships
          await get().refreshUser()
        } catch (e) {
          set({
            error: e instanceof Error ? e.message : "Invalid magic link",
            isLoading: false,
          })
          throw e
        }
      },

      // Logout
      logout: async () => {
        const { session } = get()
        
        if (session?.token) {
          try {
            await apiLogout(session.token)
          } catch {
            // Ignore logout errors
          }
        }
        
        set({
          user: null,
          session: null,
          memberships: [],
          isAuthenticated: false,
          needsFamily: false,
          pendingInvitation: null,
        })
      },

      // Refresh user data
      refreshUser: async () => {
        const { session } = get()
        
        if (!session?.token) {
          return
        }
        
        try {
          const response = await apiGetCurrentUser(session.token)
          
          set({
            user: response.user,
            memberships: response.memberships,
            needsFamily: response.memberships.length === 0,
            isAuthenticated: true,
          })
        } catch {
          // Session expired, logout
          set({
            user: null,
            session: null,
            memberships: [],
            isAuthenticated: false,
            needsFamily: false,
          })
        }
      },

      // Check invitation code
      checkInviteCode: async (code) => {
        const invitation = await apiGetInvitationByCode(code)
        set({ pendingInvitation: invitation })
        return invitation
      },

      // Set pending invitation
      setPendingInvitation: (invitation) => {
        set({ pendingInvitation: invitation })
      },

      // Create a new family
      createFamily: async (name) => {
        const { session } = get()
        
        console.log("[AuthStore] createFamily called with:", name)
        console.log("[AuthStore] Session:", session ? "exists" : "null")
        
        if (!session?.token) {
          console.error("[AuthStore] No session token!")
          throw new Error("Not authenticated")
        }
        
        console.log("[AuthStore] Calling API to create family...")
        const family = await apiCreateFamily(session.token, name)
        console.log("[AuthStore] Family created:", family)
        
        // Refresh user to get new membership
        console.log("[AuthStore] Refreshing user...")
        await get().refreshUser()
        console.log("[AuthStore] User refreshed, needsFamily:", get().needsFamily)
        
        return family
      },

      // Clear error
      clearError: () => {
        set({ error: null })
      },

      // Initialize - check if we have a valid session
      initialize: async () => {
        const { session } = get()
        
        if (!session?.token) {
          return
        }
        
        // Check if session is expired
        if (new Date(session.expires_at) < new Date()) {
          set({
            user: null,
            session: null,
            memberships: [],
            isAuthenticated: false,
          })
          return
        }
        
        // Refresh user data
        await get().refreshUser()
      },
    }),
    {
      name: "familiar-auth",
      partialize: (state) => ({
        session: state.session,
        user: state.user,
      }),
    }
  )
)

// ============================================================================
// Selectors
// ============================================================================

export const selectIsAuthenticated = (s: AuthStore) => s.isAuthenticated
export const selectUser = (s: AuthStore) => s.user
export const selectMemberships = (s: AuthStore) => s.memberships
export const selectNeedsFamily = (s: AuthStore) => s.needsFamily
export const selectPrimaryTenant = (s: AuthStore) => 
  s.memberships.find(m => m.is_primary) || s.memberships[0]


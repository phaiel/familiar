/**
 * Settings Store
 * 
 * Manages user preferences including flow selection and tenant context.
 * Persists to localStorage for session continuity.
 */
import { create } from "zustand"
import { persist } from "zustand/middleware"

// ============================================================================
// Constants
// ============================================================================

// Default tenant (created in DB migration)
export const DEFAULT_TENANT_ID = "00000000-0000-0000-0000-000000000001"
export const DEFAULT_TENANT_NAME = "Default Family"

// ============================================================================
// Flow Configuration
// ============================================================================

export interface FlowOption {
  id: string
  name: string
  path: string
  description: string
}

// Available flows - can be extended dynamically
// Note: paths must match the Windmill flow paths (including f/ prefix for folder flows)
export const AVAILABLE_FLOWS: FlowOption[] = [
  {
    id: "agentic",
    name: "Agentic (LlamaIndex)",
    path: "f/agentic/main",
    description: "Multi-agent orchestration with LlamaIndex",
  },
]

// ============================================================================
// Store Types
// ============================================================================

interface SettingsStore {
  // Tenant context
  tenantId: string
  tenantName: string
  currentUserId: string | null
  currentUserName: string | null
  
  // Flow selection
  selectedFlowId: string
  selectedFlowPath: string
  
  // Actions
  setTenant: (tenantId: string, tenantName: string) => void
  setCurrentUser: (userId: string | null, userName: string | null) => void
  setSelectedFlow: (flowId: string) => void
  getSelectedFlow: () => FlowOption | undefined
  
  // Settings panel state
  isSettingsOpen: boolean
  setSettingsOpen: (open: boolean) => void
  toggleSettings: () => void
}

// ============================================================================
// Store Implementation
// ============================================================================

export const useSettingsStore = create<SettingsStore>()(
  persist(
    (set, get) => ({
      // Tenant context (default values)
      tenantId: DEFAULT_TENANT_ID,
      tenantName: DEFAULT_TENANT_NAME,
      currentUserId: null,
      currentUserName: null,
      
      // Default to Agentic (LlamaIndex flow)
      selectedFlowId: "agentic",
      selectedFlowPath: "f/agentic/main",
      
      setTenant: (tenantId: string, tenantName: string) => {
        set({ tenantId, tenantName })
      },
      
      setCurrentUser: (userId: string | null, userName: string | null) => {
        set({ currentUserId: userId, currentUserName: userName })
      },
      
      setSelectedFlow: (flowId: string) => {
        const flow = AVAILABLE_FLOWS.find(f => f.id === flowId)
        if (flow) {
          set({
            selectedFlowId: flowId,
            selectedFlowPath: flow.path,
          })
        }
      },
      
      getSelectedFlow: () => {
        const { selectedFlowId } = get()
        return AVAILABLE_FLOWS.find(f => f.id === selectedFlowId)
      },
      
      isSettingsOpen: false,
      setSettingsOpen: (open: boolean) => set({ isSettingsOpen: open }),
      toggleSettings: () => set((s) => ({ isSettingsOpen: !s.isSettingsOpen })),
    }),
    {
      name: "familiar-settings",
      version: 2, // Bump version to trigger migration
      partialize: (state) => ({
        tenantId: state.tenantId,
        tenantName: state.tenantName,
        currentUserId: state.currentUserId,
        currentUserName: state.currentUserName,
        selectedFlowId: state.selectedFlowId,
        selectedFlowPath: state.selectedFlowPath,
      }),
      // Migrate old flow paths to new format
      migrate: (persistedState: any, version: number) => {
        if (version < 2) {
          // Fix old flow paths (u/phaiel/loom → f/agentic/main)
          const oldPath = persistedState?.selectedFlowPath
          if (oldPath && (oldPath.startsWith('u/') || !oldPath.startsWith('f/'))) {
            console.log('[SettingsStore] Migrating flow path:', oldPath, '→ f/agentic/main')
            persistedState.selectedFlowId = 'agentic'
            persistedState.selectedFlowPath = 'f/agentic/main'
          }
        }
        return persistedState
      },
    }
  )
)

// Convenience selectors
export const selectFlowPath = (s: SettingsStore) => s.selectedFlowPath
export const selectTenantId = (s: SettingsStore) => s.tenantId


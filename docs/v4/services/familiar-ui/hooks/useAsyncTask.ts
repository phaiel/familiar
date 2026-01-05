/**
 * useAsyncTask - Hook for polling async task status
 * 
 * Used by onboarding flows that trigger Windmill DAGs via the async task API.
 * Polls the task endpoint until completion or failure.
 */
import { useState, useEffect, useCallback, useRef } from "react"

// ============================================================================
// Types
// ============================================================================

export type TaskStatus = 
  | "pending" 
  | "queued" 
  | "running" 
  | "completed" 
  | "failed" 
  | "cancelled"

export interface AsyncTask {
  task_id: string
  status: TaskStatus
  task_type: string
  correlation_id: string
  output?: Record<string, unknown>
  error_message?: string
  attempt_count: number
  created_at: string
  started_at?: string
  completed_at?: string
}

export interface CreateTaskInput {
  task_type: string
  input: Record<string, unknown>
  tenant_id?: string
}

export interface CreateTaskResponse {
  task_id: string
  correlation_id: string
  status: TaskStatus
  poll_url: string
}

// ============================================================================
// API Functions
// ============================================================================

const API_BASE = "/api"

async function getAuthToken(): Promise<string | null> {
  // Get token from auth store - assuming it's stored in localStorage
  const authData = localStorage.getItem("familiar-auth")
  if (!authData) return null
  
  try {
    const parsed = JSON.parse(authData)
    return parsed.state?.session?.token || null
  } catch {
    return null
  }
}

async function fetchTask(taskId: string): Promise<AsyncTask> {
  const token = await getAuthToken()
  
  const res = await fetch(`${API_BASE}/tasks/${taskId}`, {
    headers: token ? { Authorization: `Bearer ${token}` } : {},
  })
  
  if (!res.ok) {
    const error = await res.json().catch(() => ({ error: "Unknown error" }))
    throw new Error(error.error || `Failed to fetch task: ${res.status}`)
  }
  
  return res.json()
}

async function createTask(input: CreateTaskInput): Promise<CreateTaskResponse> {
  const token = await getAuthToken()
  
  if (!token) {
    throw new Error("Not authenticated")
  }
  
  const res = await fetch(`${API_BASE}/tasks`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(input),
  })
  
  if (!res.ok) {
    const error = await res.json().catch(() => ({ error: "Unknown error" }))
    throw new Error(error.error || `Failed to create task: ${res.status}`)
  }
  
  return res.json()
}

// ============================================================================
// Hook
// ============================================================================

export interface UseAsyncTaskOptions {
  /** Polling interval in milliseconds (default: 1000) */
  pollInterval?: number
  /** Maximum number of poll attempts before giving up (default: 300 = 5 minutes) */
  maxAttempts?: number
  /** Callback when task completes successfully */
  onSuccess?: (task: AsyncTask) => void
  /** Callback when task fails */
  onError?: (error: Error, task?: AsyncTask) => void
}

export interface UseAsyncTaskReturn {
  /** Current task state */
  task: AsyncTask | null
  /** Whether we're currently polling */
  isPolling: boolean
  /** Any error that occurred */
  error: Error | null
  /** Start tracking a task by ID */
  startPolling: (taskId: string) => void
  /** Stop polling */
  stopPolling: () => void
  /** Create and start tracking a new task */
  createAndPoll: (input: CreateTaskInput) => Promise<string>
  /** Reset the hook state */
  reset: () => void
}

export function useAsyncTask(options: UseAsyncTaskOptions = {}): UseAsyncTaskReturn {
  const {
    pollInterval = 1000,
    maxAttempts = 300,
    onSuccess,
    onError,
  } = options

  const [task, setTask] = useState<AsyncTask | null>(null)
  const [isPolling, setIsPolling] = useState(false)
  const [error, setError] = useState<Error | null>(null)
  
  const taskIdRef = useRef<string | null>(null)
  const attemptCountRef = useRef(0)
  const timeoutRef = useRef<NodeJS.Timeout | null>(null)

  const stopPolling = useCallback(() => {
    setIsPolling(false)
    taskIdRef.current = null
    attemptCountRef.current = 0
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current)
      timeoutRef.current = null
    }
  }, [])

  const poll = useCallback(async () => {
    if (!taskIdRef.current) return
    
    attemptCountRef.current += 1
    
    if (attemptCountRef.current > maxAttempts) {
      const err = new Error("Polling timeout - task took too long")
      setError(err)
      stopPolling()
      onError?.(err, task ?? undefined)
      return
    }
    
    try {
      const updatedTask = await fetchTask(taskIdRef.current)
      setTask(updatedTask)
      
      // Check terminal states
      if (updatedTask.status === "completed") {
        stopPolling()
        onSuccess?.(updatedTask)
        return
      }
      
      if (updatedTask.status === "failed" || updatedTask.status === "cancelled") {
        const err = new Error(updatedTask.error_message || `Task ${updatedTask.status}`)
        setError(err)
        stopPolling()
        onError?.(err, updatedTask)
        return
      }
      
      // Continue polling
      timeoutRef.current = setTimeout(poll, pollInterval)
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err))
      setError(error)
      stopPolling()
      onError?.(error, task ?? undefined)
    }
  }, [maxAttempts, pollInterval, onSuccess, onError, stopPolling, task])

  const startPolling = useCallback((taskId: string) => {
    // Reset state
    setTask(null)
    setError(null)
    attemptCountRef.current = 0
    
    // Start polling
    taskIdRef.current = taskId
    setIsPolling(true)
    poll()
  }, [poll])

  const createAndPoll = useCallback(async (input: CreateTaskInput): Promise<string> => {
    setError(null)
    
    try {
      const response = await createTask(input)
      startPolling(response.task_id)
      return response.task_id
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err))
      setError(error)
      throw error
    }
  }, [startPolling])

  const reset = useCallback(() => {
    stopPolling()
    setTask(null)
    setError(null)
  }, [stopPolling])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current)
      }
    }
  }, [])

  return {
    task,
    isPolling,
    error,
    startPolling,
    stopPolling,
    createAndPoll,
    reset,
  }
}

// ============================================================================
// Convenience Hooks for Specific Flows
// ============================================================================

/**
 * Hook for the signup flow
 */
export function useSignupTask(options?: Omit<UseAsyncTaskOptions, "onSuccess" | "onError">) {
  const [result, setResult] = useState<{
    user_id: string
    session_token: string
    needs_family: boolean
    joined_family_id?: string
  } | null>(null)

  const taskHook = useAsyncTask({
    ...options,
    onSuccess: (task) => {
      if (task.output) {
        setResult(task.output as typeof result)
      }
    },
  })

  const signup = useCallback(async (input: {
    email: string
    password?: string
    name: string
    invite_code?: string
    consents: { terms: boolean; privacy: boolean }
  }) => {
    return taskHook.createAndPoll({
      task_type: "onboarding.signup",
      input: {
        ...input,
        request_id: crypto.randomUUID(),
      },
    })
  }, [taskHook])

  return {
    ...taskHook,
    result,
    signup,
  }
}

/**
 * Hook for the create family flow
 */
export function useCreateFamilyTask(options?: Omit<UseAsyncTaskOptions, "onSuccess" | "onError">) {
  const [result, setResult] = useState<{
    tenant_id: string
    tenant_name: string
    channel_id: string
  } | null>(null)

  const taskHook = useAsyncTask({
    ...options,
    onSuccess: (task) => {
      if (task.output) {
        setResult(task.output as typeof result)
      }
    },
  })

  const createFamily = useCallback(async (input: {
    user_id: string
    family_name: string
  }) => {
    return taskHook.createAndPoll({
      task_type: "onboarding.create_family",
      input: {
        ...input,
        request_id: crypto.randomUUID(),
      },
    })
  }, [taskHook])

  return {
    ...taskHook,
    result,
    createFamily,
  }
}

/**
 * Hook for accepting an invitation
 */
export function useAcceptInvitationTask(options?: Omit<UseAsyncTaskOptions, "onSuccess" | "onError">) {
  const [result, setResult] = useState<{
    tenant_id: string
    tenant_name: string
    role: string
  } | null>(null)

  const taskHook = useAsyncTask({
    ...options,
    onSuccess: (task) => {
      if (task.output) {
        setResult(task.output as typeof result)
      }
    },
  })

  const acceptInvitation = useCallback(async (input: {
    user_id: string
    invite_code: string
  }) => {
    return taskHook.createAndPoll({
      task_type: "onboarding.accept_invitation",
      input: {
        ...input,
        request_id: crypto.randomUUID(),
      },
    })
  }, [taskHook])

  return {
    ...taskHook,
    result,
    acceptInvitation,
  }
}





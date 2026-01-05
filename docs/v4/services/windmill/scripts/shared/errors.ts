// Standardized Error Format for Windmill Scripts
// Use this in ALL Deno and TypeScript scripts

/**
 * Standard error envelope format
 * Consistent across Deno and Python scripts
 */
export interface FamiliarError {
  /** Always true for errors */
  error: true;
  /** Machine-readable error code (e.g., 'USER_NOT_FOUND') */
  code: string;
  /** Human-readable error message */
  message: string;
  /** Optional additional details (validation errors, etc.) */
  details?: unknown;
}

/**
 * Create a standardized error object
 * 
 * @example
 * throw new Error(JSON.stringify(familiarError('USER_NOT_FOUND', 'User does not exist')));
 * // Or in a flow:
 * return familiarError('VALIDATION_ERROR', 'Invalid input', errors);
 */
export function familiarError(code: string, message: string, details?: unknown): FamiliarError {
  return {
    error: true,
    code,
    message,
    details,
  };
}

/**
 * Throw a standardized error
 * 
 * @example
 * throwFamiliarError('EMAIL_EXISTS', 'An account with this email already exists');
 */
export function throwFamiliarError(code: string, message: string, details?: unknown): never {
  throw new Error(JSON.stringify(familiarError(code, message, details)));
}

// Common error codes
export const ErrorCodes = {
  // Validation
  VALIDATION_ERROR: 'VALIDATION_ERROR',
  INVALID_INPUT: 'INVALID_INPUT',
  
  // Authentication
  UNAUTHORIZED: 'UNAUTHORIZED',
  FORBIDDEN: 'FORBIDDEN',
  
  // User/Entity not found
  USER_NOT_FOUND: 'USER_NOT_FOUND',
  TENANT_NOT_FOUND: 'TENANT_NOT_FOUND',
  ENTITY_NOT_FOUND: 'ENTITY_NOT_FOUND',
  
  // User state
  EMAIL_EXISTS: 'EMAIL_EXISTS',
  ALREADY_MEMBER: 'ALREADY_MEMBER',
  ALREADY_HAS_FAMILY: 'ALREADY_HAS_FAMILY',
  
  // Invitations
  INVALID_CODE: 'INVALID_CODE',
  EXPIRED: 'EXPIRED',
  LIMIT_REACHED: 'LIMIT_REACHED',
  
  // Database
  DATABASE_ERROR: 'DATABASE_ERROR',
  CONSTRAINT_VIOLATION: 'CONSTRAINT_VIOLATION',
  
  // External services
  KAFKA_ERROR: 'KAFKA_ERROR',
  LLM_ERROR: 'LLM_ERROR',
  
  // Generic
  INTERNAL_ERROR: 'INTERNAL_ERROR',
  NOT_IMPLEMENTED: 'NOT_IMPLEMENTED',
} as const;

export type ErrorCode = typeof ErrorCodes[keyof typeof ErrorCodes];








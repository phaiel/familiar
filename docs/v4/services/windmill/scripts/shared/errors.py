"""Standardized Error Format for Windmill Scripts

Use this in ALL Python scripts for consistent error handling.

Example:
    from shared.errors import familiar_error, throw_familiar_error, ErrorCodes

    # Return an error dict
    return familiar_error(ErrorCodes.USER_NOT_FOUND, "User does not exist")

    # Or raise an exception
    throw_familiar_error(ErrorCodes.VALIDATION_ERROR, "Invalid input", details=errors)
"""

from typing import Any, Optional, TypedDict, NoReturn
from enum import Enum


class FamiliarError(TypedDict):
    """Standard error envelope format - consistent across Deno and Python scripts"""
    error: bool  # Always True for errors
    code: str  # Machine-readable error code
    message: str  # Human-readable message
    details: Optional[Any]  # Additional context


def familiar_error(code: str, message: str, details: Any = None) -> FamiliarError:
    """Create a standardized error dict.
    
    Args:
        code: Machine-readable error code (e.g., 'USER_NOT_FOUND')
        message: Human-readable error message
        details: Optional additional context (validation errors, etc.)
    
    Returns:
        FamiliarError dict suitable for JSON response
    
    Example:
        return familiar_error("USER_NOT_FOUND", "User does not exist")
    """
    return {
        "error": True,
        "code": code,
        "message": message,
        "details": details,
    }


def throw_familiar_error(code: str, message: str, details: Any = None) -> NoReturn:
    """Raise a standardized error as ValueError.
    
    Args:
        code: Machine-readable error code
        message: Human-readable error message
        details: Optional additional context
    
    Raises:
        ValueError with JSON-serializable error dict
    
    Example:
        throw_familiar_error("EMAIL_EXISTS", "An account with this email exists")
    """
    import json
    raise ValueError(json.dumps(familiar_error(code, message, details)))


class ErrorCodes:
    """Common error codes - use these instead of magic strings"""
    
    # Validation
    VALIDATION_ERROR = "VALIDATION_ERROR"
    INVALID_INPUT = "INVALID_INPUT"
    
    # Authentication
    UNAUTHORIZED = "UNAUTHORIZED"
    FORBIDDEN = "FORBIDDEN"
    
    # User/Entity not found
    USER_NOT_FOUND = "USER_NOT_FOUND"
    TENANT_NOT_FOUND = "TENANT_NOT_FOUND"
    ENTITY_NOT_FOUND = "ENTITY_NOT_FOUND"
    
    # User state
    EMAIL_EXISTS = "EMAIL_EXISTS"
    ALREADY_MEMBER = "ALREADY_MEMBER"
    ALREADY_HAS_FAMILY = "ALREADY_HAS_FAMILY"
    
    # Invitations
    INVALID_CODE = "INVALID_CODE"
    EXPIRED = "EXPIRED"
    LIMIT_REACHED = "LIMIT_REACHED"
    
    # Database
    DATABASE_ERROR = "DATABASE_ERROR"
    CONSTRAINT_VIOLATION = "CONSTRAINT_VIOLATION"
    
    # External services
    KAFKA_ERROR = "KAFKA_ERROR"
    LLM_ERROR = "LLM_ERROR"
    
    # Generic
    INTERNAL_ERROR = "INTERNAL_ERROR"
    NOT_IMPLEMENTED = "NOT_IMPLEMENTED"








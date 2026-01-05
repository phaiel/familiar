"""
Global State Management for Agentic System

Manages the shared state across all agents in the orchestration loop.
State is passed between agents and Windmill flow steps.

## Schema-First Architecture

The canonical schemas live in `familiar-schemas/`. For Windmill:

1. **Option A (Recommended)**: Use Windmill Resources
   - Store JSON schemas in Windmill Resources
   - Validate at runtime using `jsonschema` library

2. **Option B**: Bundle types in a pip package  
   - Publish `familiar-types` to private PyPI
   - Add to Windmill workspace requirements.txt

3. **Option C (Current)**: Inline fallback types
   - Types defined below as fallback when generated_pydantic unavailable
   - MUST be manually kept in sync with familiar-schemas

See: https://www.windmill.dev/docs/advanced/imports
"""
from typing import Dict, Any, Optional, Protocol, List
from datetime import datetime
from pydantic import BaseModel

# Try to import from bundled package (Option B) if available
# Falls back to inline definitions (Option C) if not installed
try:
    from familiar_types.agentic import AgentState, ConversationTurn
except ImportError:
    # Inline fallback types - keep in sync with familiar-schemas
    # Run `cargo xtask schemas validate` to check drift
    
    class ConversationTurn(BaseModel):
        """A single turn in a conversation.
        
        Schema: familiar-schemas/versions/v0.7.0/json-schema/agentic/ConversationTurn.json
        """
        content: str
        role: str
        speaker: Optional[str] = None
        timestamp: Optional[str] = None

    class AgentState(BaseModel):
        """Global state shared across agents.
        
        Schema: familiar-schemas/versions/v0.7.0/json-schema/agentic/AgentState.json
        """
        conversation_context: List[ConversationTurn] = []
        current_speaker: Optional[str] = None
        is_authenticated: bool = False
        just_finished: bool = False
        metadata: Optional[Dict[str, Any]] = None
        tenant_id: str
        thread_id: Optional[str] = None


# Agent module protocol for extensibility
class AgentModule(Protocol):
    """Interface for pluggable agent modules (guardrails.ai, physics, etc.)"""
    
    def process(self, state: AgentState, input: Dict[str, Any]) -> Dict[str, Any]:
        """Process input and return output"""
        ...
    
    def validate(self, data: Dict[str, Any]) -> bool:
        """Validate data before/after processing"""
        ...


# Module registry for future extensibility
REGISTERED_MODULES: Dict[str, AgentModule] = {
    # Future modules:
    # "guardrails": GuardrailsModule(),
    # "physics": PhysicsModule(),
}


def create_initial_state(tenant_id: str, thread_id: Optional[str] = None) -> AgentState:
    """Create initial state for a new conversation"""
    return AgentState(
        tenant_id=tenant_id,
        thread_id=thread_id,
        current_speaker=None,
        is_authenticated=False,
        just_finished=False,
        conversation_context=[],
        metadata={},
    )


def add_turn(state: AgentState, role: str, content: str) -> AgentState:
    """Add a conversation turn to the state"""
    turn = ConversationTurn(
        role=role,
        content=content,
        speaker=state.current_speaker,
        timestamp=datetime.utcnow().isoformat() + "Z",
    )
    state.conversation_context.append(turn)
    return state


def set_speaker(state: AgentState, speaker: Optional[str]) -> AgentState:
    """Set the current speaker"""
    state.current_speaker = speaker
    return state


def finish_task(state: AgentState) -> AgentState:
    """Mark the current task as finished"""
    state.just_finished = True
    state.current_speaker = None
    return state


def reset_finished(state: AgentState) -> AgentState:
    """Reset the just_finished flag"""
    state.just_finished = False
    return state


def serialize_state(state: AgentState) -> Dict[str, Any]:
    """Serialize state for Windmill flow passing"""
    return state.model_dump()


def deserialize_state(data: Dict[str, Any]) -> AgentState:
    """Deserialize state from Windmill flow"""
    return AgentState.model_validate(data)


def validate_with_module(module_name: str, data: Dict[str, Any]) -> bool:
    """Validate data using a registered module"""
    if module_name not in REGISTERED_MODULES:
        return True  # No validation if module not registered
    return REGISTERED_MODULES[module_name].validate(data)

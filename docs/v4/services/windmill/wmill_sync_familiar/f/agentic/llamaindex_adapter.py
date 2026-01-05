"""Adapters between LlamaIndex runtime objects and familiar-core schema models.

Schema-first: all structures must conform to the production weave result format.
Uses Fates-themed agents: Gate, Morta, Decima, Nona.

## Windmill Environment

This runs in Windmill's sandboxed environment. Types are:
1. Imported from `familiar_types` pip package (if installed)
2. Defined inline as fallback (must match familiar-schemas)

See state.py for the canonical schema import pattern.
"""
from typing import Any, Dict, Optional, List
from datetime import datetime
import uuid

from pydantic import BaseModel, Field
from enum import Enum

# Schema-first types - try bundled package, fall back to inline
try:
    from familiar_types.agentic import AgentState, AgentResponse, ConversationTurn
    USING_GENERATED = True
except ImportError:
    USING_GENERATED = False
    # Inline fallback types - keep in sync with familiar-schemas

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

    class AgentResponse(BaseModel):
        """Fallback - matches generated/python/agentic/AgentResponse_schema.py"""
        message_type: Optional[str] = "text"
        content: str
        options: Optional[List[str]] = None


# ---------------------------------------------------------------------------
# Fates Agent Names
# ---------------------------------------------------------------------------

class FatesAgent(str, Enum):
    """The Fates - agents in the weaving pipeline"""
    GATE = "gate"      # Intent detection, routing
    MORTA = "morta"    # Segmentation (cuts the thread)
    DECIMA = "decima"  # Classification/measurement (measures the thread)
    NONA = "nona"      # Response generation (spins the thread)


# ---------------------------------------------------------------------------
# UI Schema Models (Pydantic equivalents of TypeScript generated types)
# These match: familiar-core/generated/typescript/UI*.ts
# TODO: Generate these from familiar-core Rust types when schemars->Pydantic pipeline is ready
# ---------------------------------------------------------------------------

class ToolCallStatus(str, Enum):
    """Status of a tool call - matches ToolCallStatus.ts"""
    PENDING = "pending"
    RUNNING = "running"
    COMPLETE = "complete"
    ERROR = "error"


class UIThinkingStep(BaseModel):
    """A thinking/reasoning step - matches UIThinkingStep.ts"""
    id: str
    agent: str
    thought: str
    timestamp: Optional[str] = None


class UIToolCall(BaseModel):
    """A tool call with status updates - matches UIToolCall.ts"""
    id: str
    tool: str
    arguments: Optional[Dict[str, Any]] = None
    result: Optional[Any] = None
    status: ToolCallStatus = ToolCallStatus.PENDING


# ---------------------------------------------------------------------------
# Weave Result Schema (Pydantic equivalents of TypeScript generated types)
# These match: familiar-core/generated/typescript/Weave*.ts, Spawn*.ts
# TODO: Generate these from familiar-core Rust types
# ---------------------------------------------------------------------------

class WeavePhysics(BaseModel):
    """Physics dimensions - matches PhysicsHint.ts / WeavePhysics fields"""
    valence: float = 0.0
    arousal: float = 0.0
    significance: float = 0.0
    epistemic: float = 0.0  # How much new information


class WeaveEntity(BaseModel):
    """Entity classification - matches entity type patterns in familiar-core"""
    type: str  # MOMENT, THREAD, INTENT, NOTE, TASK, etc.
    probability: float


class WeaveUnit(BaseModel):
    """A single unit of woven content - matches WeaveUnit.ts"""
    index: int
    content: str
    physics: WeavePhysics
    subject: Optional[str] = None
    entities: List[WeaveEntity] = []
    unit_type: str = "content"
    subject_type: Optional[str] = None
    gathered_from: List[str] = []
    threads_referenced: List[str] = []


class SpawnSuggestion(BaseModel):
    """A suggestion to spawn a new entity - matches SpawnSuggestion.ts"""
    action: str = "suggest"  # suggest, auto_spawn
    reason: str
    content: str
    physics: WeavePhysics
    subject: Optional[str] = None
    confidence: float
    entity_type: str
    weave_unit_index: int


class SpawnSummary(BaseModel):
    """Summary of spawn suggestions - matches SpawnSummary.ts"""
    review_count: int = 0
    auto_spawn_count: int = 0


class SpawnResult(BaseModel):
    """Spawn suggestions from classification - matches SpawnResult patterns"""
    summary: SpawnSummary = Field(default_factory=SpawnSummary)
    suggestions: List[SpawnSuggestion] = []


class WeaveResult(BaseModel):
    """
    Production weave result format - matches WeaveResult patterns.
    
    This replaces the old heddle_result with a richer structure
    that includes weave_units, intent, spawn suggestions, etc.
    """
    intent: str = "QUERY"  # QUERY, LOG, COMMAND, etc.
    weave_units: List[WeaveUnit] = []
    spawn: SpawnResult = Field(default_factory=SpawnResult)
    processed_at: str = ""
    primary_theme: Optional[str] = None
    subjects_identified: List[str] = []


def ensure_agent_state(data: Dict[str, Any]) -> AgentState:
    """Validate and normalize incoming state dict to AgentState."""
    return AgentState.model_validate(data)


def make_turn(role: str, content: str, speaker: Optional[str] = None) -> ConversationTurn:
    return ConversationTurn(
        role=role,
        content=content,
        speaker=speaker,
        timestamp=datetime.utcnow().isoformat() + "Z",
    )


def make_agent_response(content: str, message_type: str = "text", options: Optional[List[str]] = None) -> AgentResponse:
    return AgentResponse(
        content=content,
        message_type=message_type,
        options=options,
    )


def make_thinking_step(agent: str, thought: str, emoji: str = "") -> Dict[str, Any]:
    """Create a UIThinkingStep-compliant dict using Pydantic validation."""
    step = UIThinkingStep(
        id=str(uuid.uuid4()),
        agent=agent,
        thought=f"{emoji} {thought}".strip() if emoji else thought,
        timestamp=datetime.utcnow().isoformat() + "Z",
    )
    return step.model_dump()


def make_tool_call(
    tool: str, 
    arguments: Optional[Dict[str, Any]] = None, 
    result: Optional[Any] = None, 
    status: str = "running"
) -> Dict[str, Any]:
    """Create a UIToolCall-compliant dict using Pydantic validation."""
    # Map string status to enum
    status_enum = ToolCallStatus(status) if status in [s.value for s in ToolCallStatus] else ToolCallStatus.RUNNING
    
    # Parse arguments if they're a string (JSON)
    parsed_args = arguments
    if isinstance(arguments, str):
        try:
            import json
            parsed_args = json.loads(arguments)
        except (json.JSONDecodeError, TypeError):
            parsed_args = {"raw": arguments}
    
    call = UIToolCall(
        id=str(uuid.uuid4()),
        tool=tool,
        arguments=parsed_args,
        result=result,
        status=status_enum,
    )
    return call.model_dump()


def map_stream_chunk(chunk: Dict[str, Any]) -> Dict[str, Any]:
    """
    Map a LlamaIndex streaming chunk to UI-friendly schema pieces.
    Expected chunk fields (best-effort): type, content/tool_name/arguments/result.
    """
    ctype = chunk.get("type")
    if ctype == "thinking":
        return {"thinking_steps": [make_thinking_step(agent=chunk.get("agent", "nona"), thought=chunk.get("content", ""))]}
    if ctype == "tool_call":
        return {"tool_calls": [make_tool_call(tool=chunk.get("tool_name", "tool"), arguments=chunk.get("arguments"))]}
    if ctype == "tool_result":
        return {"tool_calls": [make_tool_call(tool=chunk.get("tool_name", "tool"), result=chunk.get("result"), status="complete")]}
    return {}


# ---------------------------------------------------------------------------
# WeaveResult builders (production format)
# ---------------------------------------------------------------------------

def make_weave_physics(
    valence: float = 0.0,
    arousal: float = 0.0,
    significance: float = 0.0,
    epistemic: float = 0.0,
) -> Dict[str, Any]:
    """Create WeavePhysics dict."""
    return WeavePhysics(
        valence=valence,
        arousal=arousal,
        significance=significance,
        epistemic=epistemic,
    ).model_dump()


def make_weave_entity(entity_type: str, probability: float) -> Dict[str, Any]:
    """Create WeaveEntity dict."""
    return WeaveEntity(type=entity_type, probability=probability).model_dump()


def make_weave_unit(
    index: int,
    content: str,
    physics: Dict[str, Any],
    entities: List[Dict[str, Any]],
    subject: Optional[str] = None,
    unit_type: str = "content",
    subject_type: Optional[str] = None,
    gathered_from: Optional[List[str]] = None,
    threads_referenced: Optional[List[str]] = None,
) -> Dict[str, Any]:
    """Create WeaveUnit dict."""
    return WeaveUnit(
        index=index,
        content=content,
        physics=WeavePhysics(**physics),
        entities=[WeaveEntity(**e) for e in entities],
        subject=subject,
        unit_type=unit_type,
        subject_type=subject_type,
        gathered_from=gathered_from or [content],
        threads_referenced=threads_referenced or [],
    ).model_dump()


def make_spawn_suggestion(
    content: str,
    entity_type: str,
    confidence: float,
    physics: Dict[str, Any],
    weave_unit_index: int,
    subject: Optional[str] = None,
    action: str = "suggest",
) -> Dict[str, Any]:
    """Create SpawnSuggestion dict."""
    return SpawnSuggestion(
        action=action,
        reason=f"{entity_type} for \"content\" at {int(confidence * 100)}%",
        content=content,
        physics=WeavePhysics(**physics),
        subject=subject,
        confidence=confidence,
        entity_type=entity_type,
        weave_unit_index=weave_unit_index,
    ).model_dump()


def make_weave_result(
    intent: str = "QUERY",
    weave_units: Optional[List[Dict[str, Any]]] = None,
    spawn_suggestions: Optional[List[Dict[str, Any]]] = None,
    primary_theme: Optional[str] = None,
    subjects_identified: Optional[List[str]] = None,
) -> Dict[str, Any]:
    """
    Create a production WeaveResult dict.
    
    This is the new format replacing heddle_result.
    """
    review_count = len(spawn_suggestions) if spawn_suggestions else 0
    
    return WeaveResult(
        intent=intent,
        weave_units=[WeaveUnit(**u) for u in (weave_units or [])],
        spawn=SpawnResult(
            summary=SpawnSummary(review_count=review_count, auto_spawn_count=0),
            suggestions=[SpawnSuggestion(**s) for s in (spawn_suggestions or [])],
        ),
        processed_at=datetime.utcnow().isoformat() + "Z",
        primary_theme=primary_theme or intent.lower(),
        subjects_identified=subjects_identified or [],
    ).model_dump()


def aggregate_weave_from_tool_results(
    content: str,
    tool_results: List[Dict[str, Any]],
    intent: str = "QUERY",
) -> Dict[str, Any]:
    """
    Aggregate tool call results into a WeaveResult.
    
    Looks for classifier and physics tool results to build weave_units and spawn suggestions.
    """
    weave_units: List[Dict[str, Any]] = []
    spawn_suggestions: List[Dict[str, Any]] = []
    subjects_identified: List[str] = []
    
    # Default physics
    physics = make_weave_physics(valence=0.0, arousal=0.4, significance=0.3, epistemic=0.5)
    entities: List[Dict[str, Any]] = []
    
    for tool in tool_results:
        tool_name = tool.get("tool", "")
        result = tool.get("result")
        if not result or not isinstance(result, dict):
            continue
            
        if tool_name in ("classifier", "decima_classify"):
            # Extract classifications as entities
            for c in result.get("classifications", []):
                entity_type = c.get("entity_type", c.get("type", "MOMENT"))
                prob = c.get("probability", c.get("confidence", 0.5))
                entities.append(make_weave_entity(entity_type, prob))
                
                # Create spawn suggestion for each classification
                spawn_suggestions.append(make_spawn_suggestion(
                    content=content,
                    entity_type=entity_type,
                    confidence=prob,
                    physics=physics,
                    weave_unit_index=0,
                ))
            
            # Update intent if provided
            if "intent" in result:
                intent = result["intent"]
                
        elif tool_name == "physics":
            # Update physics from tool result
            physics = make_weave_physics(
                valence=result.get("valence", 0.0),
                arousal=result.get("arousal", 0.4),
                significance=result.get("significance", 0.3),
                epistemic=result.get("epistemic", result.get("clarity", 0.5)),
            )
    
    # If no entities from tools, add default based on intent
    if not entities:
        if intent == "QUERY":
            entities = [make_weave_entity("INTENT", 0.7), make_weave_entity("THREAD", 0.3)]
        else:
            entities = [make_weave_entity("MOMENT", 0.8)]
    
    # Create the weave unit
    weave_units.append(make_weave_unit(
        index=0,
        content=content,
        physics=physics,
        entities=entities,
        unit_type=intent.lower(),
        subject_type=intent.lower(),
        gathered_from=[content],
    ))
    
    return make_weave_result(
        intent=intent,
        weave_units=weave_units,
        spawn_suggestions=spawn_suggestions,
        primary_theme=intent.lower(),
        subjects_identified=subjects_identified,
    )


# ---------------------------------------------------------------------------
# Legacy alias (backwards compatibility)
# ---------------------------------------------------------------------------

def aggregate_heddle_from_tool_results(tool_results: List[Dict[str, Any]]) -> Optional[Dict[str, Any]]:
    """
    DEPRECATED: Use aggregate_weave_from_tool_results instead.
    
    This is kept for backwards compatibility during migration.
    """
    # Return None to indicate no legacy heddle result
    # Callers should use weave_result instead
    return None

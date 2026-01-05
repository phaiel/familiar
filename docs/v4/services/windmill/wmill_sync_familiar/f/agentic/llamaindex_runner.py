#requirements:
#llama-index-core==0.11.23
#llama-index-llms-anthropic==0.6.3
#anthropic==0.39.0

"""
LlamaIndex-backed agentic orchestrator entrypoint for Windmill.

Uses the Fates theme:
- Gate: Intent detection and routing
- Morta: Segmentation (cuts the thread)
- Decima: Classification (measures the thread)
- Nona: Response generation (spins the thread)

Windmill remains the workflow owner (DAG, retries). This module delegates
reasoning/tool-calling to LlamaIndex, while keeping I/O schema-first.

## Windmill Execution Environment

This script runs in Windmill's sandboxed Python environment where:
- `__file__` may not be defined
- Local filesystem paths (../../..) don't exist
- Use Windmill Resources/Variables for configuration
- Use pip packages or inline types for schemas

See state.py for schema import patterns.
"""
from typing import Any, Dict, Optional, List
from datetime import datetime
import os

from pydantic import BaseModel, ValidationError

from .state import (
    deserialize_state,
    serialize_state,
    add_turn,
    finish_task,
    AgentState,
)
from .llamaindex_adapter import (
    ensure_agent_state,
    make_agent_response,
    make_thinking_step,
    make_tool_call,
    aggregate_weave_from_tool_results,
    FatesAgent,
)

# Schema-first types - try bundled package, fall back to inline
try:
    from familiar_types.agentic import ConciergeInput, ConciergeOutput, AgentResponse
except ImportError:
    # Inline fallback types - keep in sync with familiar-schemas
    
    class ConciergeInput(BaseModel):
        """Input to the concierge agent.
        
        Schema: familiar-schemas/versions/v0.7.0/json-schema/agentic/ConciergeInput.json
        """
        state: Dict[str, Any]
        user_message: Optional[str] = None
        request_id: str
        is_first_message: bool = False

    class AgentResponse(BaseModel):
        """Response from an agent.
        
        Schema: familiar-schemas/versions/v0.7.0/json-schema/agentic/AgentResponse.json
        """
        message_type: str = "text"
        content: str
        options: Optional[List[str]] = None

    class ConciergeOutput(BaseModel):
        state: Dict[str, Any]
        response: Any
        done: bool = False

# LlamaIndex imports (best-effort). Fallback to stub if unavailable.
LLAMAINDEX_IMPORT_ERROR = None
try:
    from llama_index.core.agent import ReActAgent
    from llama_index.llms.anthropic import Anthropic
    from llama_index.core.llms import ChatMessage, MessageRole
    LLAMAINDEX_AVAILABLE = True
except Exception as e:
    LLAMAINDEX_AVAILABLE = False
    LLAMAINDEX_IMPORT_ERROR = f"{type(e).__name__}: {e}"
    ChatMessage = None
    MessageRole = None
    ReActAgent = None
    Anthropic = None
    print(f"[DEBUG] LlamaIndex import failed: {LLAMAINDEX_IMPORT_ERROR}")

try:
    from .tools.llamaindex_tools import build_llamaindex_tools
except ImportError:
    def build_llamaindex_tools():
        return []


# ---------------------------------------------------------------------------
# Intent Detection (Gate)
# ---------------------------------------------------------------------------

def detect_intent(user_message: str) -> str:
    """
    Gate: Detect intent from user message.
    
    Returns: QUERY, LOG, COMMAND, or CHAT
    """
    msg_lower = user_message.lower().strip()
    
    # Check for query patterns
    if any(q in msg_lower for q in ["?", "what", "how", "why", "when", "where", "who", "can you", "do you", "are you"]):
        return "QUERY"
    
    # Check for command patterns
    if any(c in msg_lower for c in ["create", "add", "delete", "remove", "update", "set", "make"]):
        return "COMMAND"
    
    # Check for logging patterns (past tense, journaling)
    if any(l in msg_lower for l in ["i went", "i did", "i was", "yesterday", "last week", "this morning", "today i"]):
        return "LOG"
    
    # Default to chat/query
    return "QUERY"


# ---------------------------------------------------------------------------
# Agent execution
# ---------------------------------------------------------------------------

def build_chat_history(conversation_context: List[Dict[str, Any]]) -> List[Any]:
    """Convert conversation_context to LlamaIndex ChatMessage list."""
    if not LLAMAINDEX_AVAILABLE or ChatMessage is None:
        return []
    
    messages = []
    for turn in conversation_context:
        role_str = turn.get("role", "user").lower()
        content = turn.get("content", "")
        
        if role_str == "user":
            role = MessageRole.USER
        elif role_str == "assistant":
            role = MessageRole.ASSISTANT
        elif role_str == "system":
            role = MessageRole.SYSTEM
        else:
            role = MessageRole.USER
            
        messages.append(ChatMessage(role=role, content=content))
    
    return messages


def run_fates_pipeline(
    user_message: str, 
    conversation_context: Optional[List[Dict[str, Any]]] = None
) -> Dict[str, Any]:
    """
    Run the Fates pipeline with LlamaIndex.
    
    Pipeline:
    1. Gate - Intent detection
    2. Morta - Segmentation (skipped for QUERY)
    3. Decima - Classification
    4. Nona - Response generation
    """
    thinking_steps: List[Dict[str, Any]] = []
    tool_calls: List[Dict[str, Any]] = []
    
    # Gate: Detect intent
    intent = detect_intent(user_message)
    thinking_steps.append(make_thinking_step(
        agent=FatesAgent.GATE.value,
        thought=f"Intent: {intent}" + (" → Skip segmentation" if intent == "QUERY" else ""),
        emoji="📥"
    ))
    
    # Morta: Segmentation (skipped for QUERY intent)
    if intent != "QUERY":
        thinking_steps.append(make_thinking_step(
            agent=FatesAgent.MORTA.value,
            thought="✂️ Segmenting content...",
            emoji="✂️"
        ))
    else:
        thinking_steps.append(make_thinking_step(
            agent=FatesAgent.MORTA.value,
            thought=f"Morta skipping - intent is {intent}, not LOG",
            emoji="⏭️"
        ))

    if not LLAMAINDEX_AVAILABLE:
        # Fallback: simple response without LlamaIndex
        thinking_steps.append(make_thinking_step(
            agent=FatesAgent.NONA.value,
            thought="Nona responding (fallback mode)...",
            emoji="🕸️"
        ))
        
        weave_result = aggregate_weave_from_tool_results(
            content=user_message,
            tool_results=[],
            intent=intent,
        )
        
        return {
            "agent": FatesAgent.NONA.value,
            "response": f"(LlamaIndex unavailable: {LLAMAINDEX_IMPORT_ERROR}) You said: {user_message}",
            "thinking_steps": thinking_steps,
            "tool_calls": tool_calls,
            "weave_result": weave_result,
            "intent": intent,
        }

    # Use Anthropic Claude directly (more reliable than LlamaIndex adapter)
    import wmill
    import anthropic
    
    api_key = wmill.get_variable("u/phaiel/anthropic_api_key")
    client = anthropic.Anthropic(api_key=api_key)

    # Decima: Classification
    thinking_steps.append(make_thinking_step(
        agent=FatesAgent.DECIMA.value,
        thought="Decima classifying 1 unit(s)...",
        emoji="⚖️"
    ))
    
    # Execute agent (Nona)
    thinking_steps.append(make_thinking_step(
        agent=FatesAgent.NONA.value,
        thought=f"Nona responding to {intent} with 1 unit(s)...",
        emoji="🕸️"
    ))
    
    # Build messages for Anthropic
    messages = []
    for ctx in (conversation_context or []):
        role = ctx.get("role", "user")
        content = ctx.get("content", "")
        if role == "user":
            messages.append({"role": "user", "content": content})
        elif role == "assistant":
            messages.append({"role": "assistant", "content": content})
    
    # Add current message
    messages.append({"role": "user", "content": user_message})
    
    # Call Anthropic directly
    response = client.messages.create(
        model="claude-3-5-sonnet-20241022",
        max_tokens=1024,
        system="You are Nona, the Familiar assistant. You spin responses from the threads of conversation. Be warm, helpful, and concise.",
        messages=messages,
    )
    
    result_text = response.content[0].text if response.content else ""

    # Collect tool calls from agent's history
    history = getattr(agent, "chat_history", [])
    for msg in history:
        if hasattr(msg, "additional_kwargs"):
            functions = msg.additional_kwargs.get("tool_calls") or []
            for fc in functions:
                tool_name = fc.get("function", {}).get("name", "tool")
                tool_args = fc.get("function", {}).get("arguments")
                tool_calls.append(
                    make_tool_call(
                        tool=tool_name,
                        arguments=tool_args,
                        status="complete",
                    )
                )
        # Check for tool results
        if hasattr(msg, "role") and str(getattr(msg, "role", "")).lower() == "tool":
            tool_name = getattr(msg, "name", "tool")
            tool_result = getattr(msg, "content", None)
            for tc in tool_calls:
                if tc["tool"] == tool_name and tc.get("result") is None:
                    tc["result"] = tool_result
                    tc["status"] = "complete"
                    break

    # Add classification thinking step if tools were used
    if tool_calls:
        thinking_steps.append(make_thinking_step(
            agent=FatesAgent.DECIMA.value,
            thought=f"Classified {len(tool_calls)} unit(s)",
            emoji="✨"
        ))
        thinking_steps.append(make_thinking_step(
            agent=FatesAgent.DECIMA.value,
            thought=f"{len(tool_calls)} spawn suggestion(s) ready",
            emoji="🎯"
        ))

    # Final Nona step
    thinking_steps.append(make_thinking_step(
        agent=FatesAgent.NONA.value,
        thought="Response woven.",
        emoji="✅"
    ))

    # Build weave_result from tool results
    weave_result = aggregate_weave_from_tool_results(
        content=user_message,
        tool_results=tool_calls,
        intent=intent,
    )

    return {
        "agent": FatesAgent.NONA.value,
        "response": result_text,
        "thinking_steps": thinking_steps,
        "tool_calls": tool_calls,
        "weave_result": weave_result,
        "intent": intent,
    }


# ---------------------------------------------------------------------------
# Windmill entrypoint
# ---------------------------------------------------------------------------

def main(input: Dict[str, Any]) -> Dict[str, Any]:
    """
    Windmill entrypoint for LlamaIndex-backed Fates pipeline.
    """
    # Feature flag to allow controlled rollout
    flag_enabled = os.getenv("LLAMAINDEX_AGENT_ENABLED", "true").lower() in ("1", "true", "yes", "on")
    if not flag_enabled:
        return {
            "error": True,
            "code": "AGENT_DISABLED",
            "message": "LlamaIndex agent disabled",
            "details": "Set LLAMAINDEX_AGENT_ENABLED=true to enable the new agentic path."
        }

    try:
        payload = ConciergeInput.model_validate(input)
    except ValidationError as e:
        return {
            "error": True,
            "code": "VALIDATION_ERROR",
            "message": "Invalid concierge input",
            "details": e.errors()
        }

    # Deserialize and validate state
    state = ensure_agent_state(payload.state)
    user_message = payload.user_message or ""
    
    # Extract conversation context for chat history
    conversation_context = []
    if hasattr(state, "conversation_context") and state.conversation_context:
        conversation_context = [
            {"role": turn.role, "content": turn.content}
            for turn in state.conversation_context
        ]

    # Add user turn
    state = add_turn(state, "user", user_message)

    # Run Fates pipeline
    pipeline_result = run_fates_pipeline(user_message, conversation_context)

    # Build response
    response = make_agent_response(pipeline_result["response"])

    # Add assistant turn
    state = add_turn(state, "assistant", response.content)

    # Mark done
    state = finish_task(state)

    return {
        "agent": pipeline_result["agent"],
        "state": serialize_state(state),
        "response": response.content,
        "thread_id": state.thread_id,
        "request_id": payload.request_id,
        "tool_calls": pipeline_result.get("tool_calls", []),
        "next_request": None,
        "weave_result": pipeline_result.get("weave_result"),
        "has_more_tasks": False,
        "thinking_steps": pipeline_result.get("thinking_steps", []),
    }


if __name__ == "__main__":
    demo = {
        "state": {
            "tenant_id": "demo",
            "thread_id": "test-thread-1",
            "conversation_context": [],
        },
        "user_message": "you alive?",
        "request_id": "demo-req-1",
    }
    import json
    print(json.dumps(main(demo), indent=2))


"""Contract smoke test for llamaindex_runner (Fates pipeline).

Validates that the runner returns schema-compatible output with:
- Fates agent names (gate, morta, decima, nona)
- weave_result (production format)
- thinking_steps and tool_calls
"""
from . import llamaindex_runner
from .llamaindex_adapter import (
    UIThinkingStep,
    UIToolCall,
    WeaveResult,
    WeaveUnit,
    WeavePhysics,
    WeaveEntity,
    SpawnSuggestion,
    FatesAgent,
    ToolCallStatus,
)


def test_runner_contract():
    """Test basic runner contract - all required fields present."""
    demo = {
        "state": {
            "tenant_id": "test-tenant",
            "thread_id": "test-thread-1",
            "conversation_context": [],
        },
        "user_message": "Test message",
        "request_id": "req-1",
    }

    result = llamaindex_runner.main(demo)

    assert result.get("state"), "state must be present"
    assert result.get("response"), "response must be present"
    assert result.get("agent"), "agent must be present"
    assert "thinking_steps" in result
    assert "tool_calls" in result
    assert "weave_result" in result, "weave_result field must be present"


def test_fates_agents():
    """Test that thinking steps use Fates agent names."""
    demo = {
        "state": {
            "tenant_id": "test-tenant",
            "thread_id": "test-thread-1",
            "conversation_context": [],
        },
        "user_message": "Hello",
        "request_id": "req-2",
    }

    result = llamaindex_runner.main(demo)
    thinking_steps = result.get("thinking_steps", [])
    
    # Verify Fates agents are used
    fates_agents = {FatesAgent.GATE.value, FatesAgent.MORTA.value, FatesAgent.DECIMA.value, FatesAgent.NONA.value}
    agents_used = {step.get("agent") for step in thinking_steps}
    
    # At least some steps should use Fates agents
    assert agents_used & fates_agents, f"Expected Fates agents, got: {agents_used}"
    
    # Final agent should be nona
    assert result.get("agent") == FatesAgent.NONA.value, f"Expected agent 'nona', got: {result.get('agent')}"


def test_weave_result_schema():
    """Test that weave_result matches production schema."""
    demo = {
        "state": {
            "tenant_id": "test-tenant",
            "thread_id": "test-thread-1",
            "conversation_context": [],
        },
        "user_message": "you alive?",
        "request_id": "req-3",
    }

    result = llamaindex_runner.main(demo)
    weave = result.get("weave_result")
    
    assert weave is not None, "weave_result should not be None"
    
    # Check required fields
    assert "intent" in weave, "weave_result must have intent"
    assert "weave_units" in weave, "weave_result must have weave_units"
    assert "spawn" in weave, "weave_result must have spawn"
    assert "processed_at" in weave, "weave_result must have processed_at"
    
    # Validate weave_units schema
    for unit in weave.get("weave_units", []):
        assert "index" in unit, "weave_unit must have index"
        assert "content" in unit, "weave_unit must have content"
        assert "physics" in unit, "weave_unit must have physics"
        assert "entities" in unit, "weave_unit must have entities"
        
        # Validate physics
        physics = unit.get("physics", {})
        assert "valence" in physics
        assert "arousal" in physics
        assert "significance" in physics
        assert "epistemic" in physics
        
        # Validate entities
        for entity in unit.get("entities", []):
            assert "type" in entity
            assert "probability" in entity
    
    # Validate spawn
    spawn = weave.get("spawn", {})
    assert "summary" in spawn, "spawn must have summary"
    assert "suggestions" in spawn, "spawn must have suggestions"
    
    # Validate spawn summary
    summary = spawn.get("summary", {})
    assert "review_count" in summary
    assert "auto_spawn_count" in summary


def test_thinking_steps_schema():
    """Test that thinking_steps items match UIThinkingStep schema."""
    demo = {
        "state": {
            "tenant_id": "test-tenant",
            "thread_id": "test-thread-1",
            "conversation_context": [],
        },
        "user_message": "Hello",
        "request_id": "req-4",
    }

    result = llamaindex_runner.main(demo)
    thinking_steps = result.get("thinking_steps", [])
    
    # Should have at least gate and nona steps
    assert len(thinking_steps) >= 2, "Should have at least 2 thinking steps"
    
    for step in thinking_steps:
        assert "id" in step, "thinking_step must have id"
        assert "agent" in step, "thinking_step must have agent"
        assert "thought" in step, "thinking_step must have thought"
        
        # Validate via Pydantic model
        UIThinkingStep.model_validate(step)


def test_tool_calls_schema():
    """Test that tool_calls items match UIToolCall schema."""
    demo = {
        "state": {
            "tenant_id": "test-tenant",
            "thread_id": "test-thread-1",
            "conversation_context": [],
        },
        "user_message": "Hello",
        "request_id": "req-5",
    }

    result = llamaindex_runner.main(demo)
    tool_calls = result.get("tool_calls", [])
    
    for call in tool_calls:
        assert "id" in call, "tool_call must have id"
        assert "tool" in call, "tool_call must have tool name"
        assert "status" in call, "tool_call must have status"
        
        # Validate status is a valid enum value
        assert call["status"] in [s.value for s in ToolCallStatus], \
            f"tool_call status must be valid enum value, got: {call['status']}"
        
        # Validate via Pydantic model
        UIToolCall.model_validate(call)


def test_intent_detection():
    """Test that Gate detects intent correctly."""
    # Query intent
    query_demo = {
        "state": {"tenant_id": "test", "conversation_context": []},
        "user_message": "What time is it?",
        "request_id": "req-query",
    }
    query_result = llamaindex_runner.main(query_demo)
    query_weave = query_result.get("weave_result", {})
    assert query_weave.get("intent") == "QUERY", f"Expected QUERY intent, got: {query_weave.get('intent')}"
    
    # Log intent
    log_demo = {
        "state": {"tenant_id": "test", "conversation_context": []},
        "user_message": "Yesterday I went for a run in the park",
        "request_id": "req-log",
    }
    log_result = llamaindex_runner.main(log_demo)
    log_weave = log_result.get("weave_result", {})
    assert log_weave.get("intent") == "LOG", f"Expected LOG intent, got: {log_weave.get('intent')}"


def test_conversation_history():
    """Test that conversation history is passed through."""
    demo = {
        "state": {
            "tenant_id": "test-tenant",
            "thread_id": "test-thread-1",
            "conversation_context": [
                {"role": "user", "content": "Hello"},
                {"role": "assistant", "content": "Hi there!"},
            ],
        },
        "user_message": "Remember what I said?",
        "request_id": "req-history",
    }

    result = llamaindex_runner.main(demo)
    
    # Should still return valid output with history
    assert result.get("state"), "state must be present"
    assert result.get("response"), "response must be present"
    assert result.get("weave_result"), "weave_result must be present"


if __name__ == "__main__":
    print("Running Fates pipeline contract tests...")
    
    test_runner_contract()
    print("✅ test_runner_contract passed")
    
    test_fates_agents()
    print("✅ test_fates_agents passed")
    
    test_weave_result_schema()
    print("✅ test_weave_result_schema passed")
    
    test_thinking_steps_schema()
    print("✅ test_thinking_steps_schema passed")
    
    test_tool_calls_schema()
    print("✅ test_tool_calls_schema passed")
    
    test_intent_detection()
    print("✅ test_intent_detection passed")
    
    test_conversation_history()
    print("✅ test_conversation_history passed")
    
    print("\n✅ All Fates pipeline contract tests passed!")


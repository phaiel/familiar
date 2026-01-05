"""
Agentic System Scripts for Windmill

Multi-agent orchestration following the LlamaIndex concierge pattern.
All types are imported from generated Pydantic models (schema-first).

Reference: https://www.llamaindex.ai/blog/building-a-multi-agent-concierge-system

Components:
- state.py: Global state management
- orchestrator.py: Routes to task agents based on state
- concierge.py: Main user-facing agent
- continuation.py: Chains agents for multi-step tasks
- tools/: Individual tool implementations
"""

"""
Agent Tools Package

Individual tool implementations that agents can use.
Each tool is a separate module that can be invoked by the orchestrator.

SCHEMA-FIRST: All types are generated from Rust schemas in familiar-core.
Import from generated_pydantic.agentic - fallback mirrors must match exactly.

Available Tools:
- classifier: Entity and intent classification
- physics: Physics simulation in VAE space
- rag: Retrieval-augmented generation (placeholder)
"""

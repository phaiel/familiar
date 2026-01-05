"""
Schema Analyzer - LLM-powered schema analysis for familiar-core

Uses Claude to analyze Rust structs and determine:
- Entity classification (domain entity, system entity, DTO, etc.)
- Whether migration to EntityMeta is needed
- Priority and suggested fixes

API Key Resource: u/phaiel/anthropic_windmill_codegen (anthropic resource type)
"""

import json
from typing import Optional
import wmill
from anthropic import Anthropic


def main(
    analysis_type: str,
    code: str,
    name: str,
    file_path: str,
    context: Optional[str] = None,
) -> dict:
    """
    Analyze a Rust struct for schema compliance.
    
    Args:
        analysis_type: One of entity_classification, generate_migration, prioritize_fixes, compliance_check
        code: The Rust code to analyze
        name: Name of the struct/type
        file_path: Path to the source file
        context: Additional context (conventions, target pattern, etc.)
    
    Returns:
        AnalysisResponse with classification, priority, reasoning, suggested_fix
    """
    # Get API key from Windmill resource (anthropic resource type with api_key field)
    resource = wmill.get_resource("u/phaiel/anthropic_windmill_codegen")
    api_key = resource["api_key"] if isinstance(resource, dict) else resource
    client = Anthropic(api_key=api_key)
    
    # Build the prompt based on analysis type
    if analysis_type == "entity_classification":
        system_prompt = ENTITY_CLASSIFICATION_PROMPT
        user_prompt = f"""Analyze this Rust struct from `{file_path}`:

```rust
{code}
```

{context or ''}

Should this struct use EntityMeta or SystemEntityMeta? Classify it and explain."""

    elif analysis_type == "generate_migration":
        system_prompt = MIGRATION_PROMPT
        user_prompt = f"""Generate a migration for this struct:

```rust
{code}
```

{context or ''}

Provide the FULL migrated struct with all necessary imports."""

    elif analysis_type == "compliance_check":
        system_prompt = COMPLIANCE_PROMPT
        user_prompt = f"""Check this struct for schema compliance:

```rust
{code}
```

{context or ''}

List any violations and suggested fixes."""

    else:
        return {
            "classification": None,
            "should_migrate": False,
            "priority": 0,
            "reasoning": f"Unknown analysis type: {analysis_type}",
            "suggested_fix": None,
            "dependencies": [],
            "error": f"Unknown analysis type: {analysis_type}",
        }

    # Call Claude
    try:
        response = client.messages.create(
            model="claude-sonnet-4-20250514",
            max_tokens=2048,
            system=system_prompt,
            messages=[{"role": "user", "content": user_prompt}],
        )
        
        # Parse the JSON response
        response_text = response.content[0].text
        
        # Try to extract JSON from the response
        try:
            # Handle case where Claude wraps in markdown
            if "```json" in response_text:
                json_start = response_text.find("```json") + 7
                json_end = response_text.find("```", json_start)
                response_text = response_text[json_start:json_end].strip()
            elif "```" in response_text:
                json_start = response_text.find("```") + 3
                json_end = response_text.find("```", json_start)
                response_text = response_text[json_start:json_end].strip()
            
            result = json.loads(response_text)
            return {
                "classification": result.get("classification"),
                "should_migrate": result.get("should_migrate", False),
                "priority": result.get("priority", result.get("migration_priority", 0)),
                "reasoning": result.get("reasoning", ""),
                "suggested_fix": result.get("suggested_fix"),
                "dependencies": result.get("dependencies", []),
                "error": None,
            }
        except json.JSONDecodeError:
            # If not JSON, return the text as reasoning
            return {
                "classification": None,
                "should_migrate": False,
                "priority": 0,
                "reasoning": response_text,
                "suggested_fix": None,
                "dependencies": [],
                "error": None,
            }
            
    except Exception as e:
        return {
            "classification": None,
            "should_migrate": False,
            "priority": 0,
            "reasoning": "",
            "suggested_fix": None,
            "dependencies": [],
            "error": str(e),
        }


ENTITY_CLASSIFICATION_PROMPT = """You are a Rust schema architecture expert analyzing code for the "familiar" project.

Classify structs into these categories:

1. **domain_entity**: Has id, tenant_id, created_at, updated_at → should use EntityMeta<{Type}Id>
2. **system_entity**: Has id, created_at, updated_at but NO tenant_id → should use SystemEntityMeta<{Type}Id>
   (Only for User and Tenant types)
3. **dto**: Input/output types (CreateXInput, UpdateXInput, XResponse) → do NOT use EntityMeta
4. **ui_type**: Frontend types (UI* prefix, string dates) → do NOT use EntityMeta
5. **component**: Composable pieces (Timestamps, Physics) → do NOT use EntityMeta

Respond with JSON only:
{
  "classification": "domain_entity|system_entity|dto|ui_type|component",
  "should_migrate": true/false,
  "priority": 1-5,
  "reasoning": "brief explanation",
  "suggested_fix": "code snippet or null",
  "dependencies": ["files that need updates"]
}"""


MIGRATION_PROMPT = """You are a Rust code generator. Generate migrated struct code.

Rules:
1. For EntityMeta: Replace id, tenant_id, created_at, updated_at with:
   #[serde(flatten)]
   pub meta: EntityMeta<{Type}Id>,

2. For SystemEntityMeta: Replace id, created_at, updated_at with:
   #[serde(flatten)]
   pub meta: SystemEntityMeta<{Type}Id>,

3. Add imports: use crate::types::base::{EntityMeta, SystemEntityMeta};
4. Keep ALL other fields unchanged
5. Preserve all derives and attributes

Respond with JSON:
{
  "classification": "domain_entity|system_entity",
  "should_migrate": true,
  "priority": 5,
  "reasoning": "Migration generated",
  "suggested_fix": "full rust code here",
  "dependencies": ["store.rs if entity is persisted"]
}"""


COMPLIANCE_PROMPT = """You are a Rust schema compliance checker for the "familiar" project.

Check for:
1. Raw Uuid usage where semantic primitives exist (UserId, TenantId, etc.)
2. Inline timestamp fields instead of using Timestamps component
3. Missing derives (Serialize, Deserialize, JsonSchema, TS)
4. Inconsistent naming patterns

Respond with JSON:
{
  "classification": "compliant|non_compliant",
  "should_migrate": true/false,
  "priority": 1-5,
  "reasoning": "list of violations",
  "suggested_fix": "how to fix",
  "dependencies": []
}"""
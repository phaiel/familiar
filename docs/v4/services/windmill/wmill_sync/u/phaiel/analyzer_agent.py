# requirements:
# wmill
# llama-index-core>=0.12.0
# llama-index-llms-anthropic>=0.6.0
# anthropic>=0.75.0
# httpx>=0.27.0

"""
Internal Schema Analyzer Agent - LlamaIndex + Anthropic + ast-grep MCP

This is an INTERNAL engineering tool, siloed in the familiar-eng workspace.
NOT part of the Familiar product (unlike the Fates agents in familiar workspace).

Connects to ast-grep MCP server via HTTP (same server Cursor uses) for
structural code analysis. The LLM MUST use ast-grep tools - analysis will
fail if tools are unavailable.

Reference: https://developers.llamaindex.ai/python/examples/tools/mcp/
"""
from __future__ import annotations  # Enable postponed evaluation of annotations

import asyncio
import json
from typing import Any, Dict, List, Optional, TYPE_CHECKING

import wmill

# Type checking imports (not evaluated at runtime)
if TYPE_CHECKING:
    from llama_index.core.agent.workflow import ReActAgent

# LlamaIndex imports
LLAMAINDEX_AVAILABLE = False
IMPORT_ERROR = ""

try:
    from llama_index.llms.anthropic import Anthropic
    from llama_index.core.agent.workflow import ReActAgent
    LLAMAINDEX_AVAILABLE = True
except ImportError as e:
    IMPORT_ERROR = str(e)


# ast-grep MCP HTTP bridge URL
# The bridge runs locally and spawns ast-grep MCP as subprocess (like Cursor)
# Use host.docker.internal to reach host from Docker (Windmill runs in Docker)
DEFAULT_AST_GREP_MCP_URL = "http://host.docker.internal:8000/mcp"


# System prompt that REQUIRES tool usage
ANALYZER_SYSTEM_PROMPT = """You are an internal schema analyzer for the Familiar project.

## PROJECT ARCHITECTURE (CRITICAL)
Understanding this architecture is essential for your analysis:

1. **familiar-core/** - THE CANONICAL SCHEMA LIBRARY
   - This is the source-of-truth for all types in the system
   - Types here are exported to TypeScript/Python via code generation
   - Issues here are usually simple: missing derives, wrong patterns
   - If analyzing familiar-core types: focus on struct composition patterns

2. **services/** - SERVICE LAYER (YOUR PRIMARY FOCUS)
   - services/familiar-api/ - Rust backend API
   - services/familiar-ui/ - TypeScript/React frontend
   - services/windmill/ - Python workflow scripts
   - Issues here are MORE COMPLEX: duplicate types, misuse of generated schemas
   - Types here should IMPORT from familiar-core, not redefine them

3. **generated/** - Auto-generated from familiar-core (DO NOT MODIFY)

## YOUR PRIMARY JOB
When analyzing types OUTSIDE familiar-core (services/, etc.):
- Check if the type duplicates something in familiar-core
- Suggest importing from familiar-core instead of redefining
- Identify schema drift between services and familiar-core

When analyzing types INSIDE familiar-core:
- Focus on struct composition patterns (EntityMeta, Timestamps)
- Check for proper semantic ID usage

## CRITICAL: You MUST use ast-grep tools
You have access to ast-grep MCP tools. You MUST use them for every analysis.
DO NOT provide analysis without first querying the codebase with ast-grep.

## Available ast-grep Tools (USE THESE)
- **find_code**: Search with patterns like "pub struct $NAME", "pub $FIELD: Uuid"
- **find_code_by_rule**: Complex YAML rules for relational queries (has, inside, follows)
- **dump_syntax_tree**: Understand AST structure for pattern debugging
- **test_match_code_rule**: Test a YAML rule against code before running on full codebase

## Your Analysis Process
1. FIRST: Identify if file is in familiar-core/ or services/
2. THEN: Use find_code to search for the struct and similar patterns in BOTH locations
3. THEN: Use find_code_by_rule to check for specific field combinations
4. FINALLY: Provide classification based on what you found

## Classifications
Classify structs into one of these categories:

1. **domain_entity**: Has id, tenant_id, created_at, updated_at → Use EntityMeta<{Type}Id>
2. **system_entity**: Has id, created_at, updated_at but NO tenant_id → Use SystemEntityMeta<{Type}Id>
   (Only for User, Tenant types)
3. **dto**: Input/output types (CreateXInput, UpdateXInput, XResponse) → Do NOT use EntityMeta
4. **ui_type**: Frontend types (UI* prefix, string dates) → Do NOT use EntityMeta
5. **component**: Composable pieces (Timestamps, EntityPhysics) → Do NOT use EntityMeta
6. **duplicate**: Type exists in familiar-core but is redefined in services → IMPORT instead
7. **kernel_candidate**: Function that looks like a pure, single-target transformation → Formalize as Kernel

## Kernel Detection (IMPORTANT)
A **Kernel** is a versioned, single-target transformation encoding a domain law.
Look for functions that:
- Take `&mut T` as the primary parameter (single target)
- Are pure (no World, no I/O, no global state)
- Are deterministic
- Do transformation/calculation (decay, score, merge, touch)

If you find a function matching this pattern, suggest formalizing as:
```rust
pub struct MyKernel;
impl Kernel<TargetType> for MyKernel {
    const ID: &'static str = "category.name.v1";
    const CATEGORY: &'static str = "law.category";
    fn apply(target: &mut TargetType, ctx: &KernelContext) { ... }
}
```

Categories: law.physics, law.decay, law.scoring, law.lifecycle, law.composition, law.validation

## Project Conventions
- Use semantic ID primitives: UserId, TenantId, ChannelId instead of raw Uuid
- Use #[serde(flatten)] for meta fields
- Create new ID primitives with define_uuid_id! macro
- Services should import from familiar-core, not redefine types
- Pure transformations should be Kernels, not loose functions

## Response Format
After using ast-grep tools, respond with structured JSON:
{
  "classification": "domain_entity|system_entity|dto|ui_type|component|duplicate|kernel_candidate",
  "should_migrate": true/false,
  "priority": 1-5,
  "reasoning": "Your analysis based on ast-grep queries you executed",
  "ast_grep_queries": ["list of ast-grep queries you ran"],
  "suggested_fix": "Full migrated code or import statement",
  "dependencies": ["files that would need updates"],
  "duplicate_of": "familiar-core path if this is a duplicate, null otherwise",
  "kernel_info": {
    "target_type": "Type being mutated (if kernel_candidate)",
    "suggested_id": "category.name.v1",
    "suggested_category": "law.category"
  }
}

REMEMBER: Always use ast-grep tools BEFORE providing your analysis."""


async def call_mcp_bridge(mcp_url: str, tool_name: str, args: dict) -> str:
    """Call an ast-grep MCP tool via HTTP bridge."""
    import httpx
    
    async with httpx.AsyncClient(timeout=60.0) as client:
        resp = await client.post(mcp_url, json={
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {"name": tool_name, "arguments": args}
        })
        result = resp.json()
        
        if "error" in result:
            return f"Error: {result['error']}"
        
        # Extract content from MCP response
        inner_result = result.get("result", {})
        content = inner_result.get("content", [])
        if content and isinstance(content, list):
            return content[0].get("text", str(content))
        # Check for direct result
        if "result" in inner_result:
            return str(inner_result["result"])
        return str(inner_result or "No result")


async def get_ast_grep_tools(mcp_url: str, project_path: str) -> List[Any]:
    """
    Connect to ast-grep MCP HTTP bridge and create LlamaIndex tools.
    
    The HTTP bridge runs locally and spawns ast-grep-server as a subprocess
    (same as Cursor does). Windmill connects to the bridge via HTTP since
    Docker doesn't have uvx installed.
    
    Args:
        mcp_url: HTTP URL of the ast-grep MCP bridge
        project_path: Root path of the project for ast-grep to search
    
    Returns:
        List of FunctionTools from the MCP server
    
    Raises:
        RuntimeError: If connection fails or no tools available
    """
    if not LLAMAINDEX_AVAILABLE:
        raise RuntimeError(f"LlamaIndex not available: {IMPORT_ERROR}")
    
    import httpx
    from llama_index.core.tools import FunctionTool
    
    print(f"[ast-grep MCP] Connecting to HTTP bridge: {mcp_url}")
    
    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            # List available tools
            resp = await client.post(mcp_url, json={
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list",
                "params": {}
            })
            result = resp.json()
            
            if "error" in result:
                raise RuntimeError(f"MCP error: {result['error']}")
            
            mcp_tools = result.get("result", {}).get("tools", [])
            if not mcp_tools:
                raise RuntimeError("No tools returned from MCP bridge")
            
            print(f"[ast-grep MCP] Available tools: {[t['name'] for t in mcp_tools]}")
            
    except httpx.RequestError as e:
        raise RuntimeError(
            f"Failed to connect to ast-grep MCP bridge at {mcp_url}: {e}\n"
            f"Start the bridge with: uvx --with starlette --with uvicorn python ast_grep_http_server.py"
        )
    
    # Create wrapper functions that call the MCP bridge
    tools = []
    
    # Store context for closures
    mcp_context = {"url": mcp_url, "project_path": project_path}
    
    def make_tool(tname: str, tdesc: str, ctx: dict):
        """Factory function to create tool with proper closure."""
        
        async def find_code_tool(
            pattern: str,
            language: str = "",
            max_results: int = 20,
            output_format: str = "text",
        ) -> str:
            """Find code using ast-grep pattern."""
            args = {
                "project_folder": ctx["project_path"],
                "pattern": pattern,
                "language": language,
                "max_results": max_results,
                "output_format": output_format,
            }
            return await call_mcp_bridge(ctx["url"], "find_code", args)
        
        async def find_code_by_rule_tool(
            yaml: str,
            max_results: int = 20,
            output_format: str = "text",
        ) -> str:
            """Find code using ast-grep YAML rule."""
            args = {
                "project_folder": ctx["project_path"],
                "yaml": yaml,
                "max_results": max_results,
                "output_format": output_format,
            }
            return await call_mcp_bridge(ctx["url"], "find_code_by_rule", args)
        
        async def dump_syntax_tree_tool(
            code: str,
            language: str,
            format: str = "cst",
        ) -> str:
            """Dump syntax tree of code."""
            args = {"code": code, "language": language, "format": format}
            return await call_mcp_bridge(ctx["url"], "dump_syntax_tree", args)
        
        async def test_match_code_rule_tool(
            code: str,
            yaml: str,
        ) -> str:
            """Test code against ast-grep YAML rule."""
            args = {"code": code, "yaml": yaml}
            return await call_mcp_bridge(ctx["url"], "test_match_code_rule", args)
        
        # Return the right function based on tool name
        if tname == "find_code":
            return FunctionTool.from_defaults(async_fn=find_code_tool, name=tname, description=tdesc)
        elif tname == "find_code_by_rule":
            return FunctionTool.from_defaults(async_fn=find_code_by_rule_tool, name=tname, description=tdesc)
        elif tname == "dump_syntax_tree":
            return FunctionTool.from_defaults(async_fn=dump_syntax_tree_tool, name=tname, description=tdesc)
        elif tname == "test_match_code_rule":
            return FunctionTool.from_defaults(async_fn=test_match_code_rule_tool, name=tname, description=tdesc)
        else:
            # Generic fallback
            async def generic_tool(**kwargs) -> str:
                return await call_mcp_bridge(ctx["url"], tname, kwargs)
            return FunctionTool.from_defaults(async_fn=generic_tool, name=tname, description=tdesc)
    
    for mcp_tool in mcp_tools:
        tool_name = mcp_tool["name"]
        tool_description = mcp_tool.get("description", f"ast-grep tool: {tool_name}")
        tools.append(make_tool(tool_name, tool_description, mcp_context))
    
    print(f"[ast-grep MCP] Created {len(tools)} tools")
    return tools


async def create_analyzer_agent(mcp_url: str, project_path: str) -> ReActAgent:
    """
    Create the internal analyzer agent with ast-grep MCP tools.
    
    This agent is for the familiar-eng workspace only.
    Will FAIL if ast-grep tools are not available.
    
    Connects to ast-grep MCP HTTP bridge which spawns ast-grep as subprocess
    (same as Cursor does, but accessible via HTTP for Docker).
    
    Args:
        mcp_url: HTTP URL of the ast-grep MCP bridge
        project_path: Root path of the project for ast-grep to search
    
    Returns:
        ReActAgent configured with ast-grep tools
    
    Raises:
        RuntimeError: If tools unavailable or API key missing
    """
    if not LLAMAINDEX_AVAILABLE:
        raise RuntimeError(f"LlamaIndex not available: {IMPORT_ERROR}")
    
    # Get Anthropic API key from Windmill resource
    try:
        resource = wmill.get_resource("u/phaiel/anthropic_windmill_codegen")
        if isinstance(resource, dict):
            api_key = resource.get("api_key")
        else:
            api_key = resource
        
        if not api_key:
            raise RuntimeError("API key is empty")
            
    except Exception as e:
        raise RuntimeError(f"Failed to get Anthropic API key from Windmill: {e}")
    
    # Create Anthropic LLM via LlamaIndex (NOT direct SDK)
    # Using Claude Haiku 4.5 - fast with excellent tool use
    llm = Anthropic(
        model="claude-haiku-4-5-20251001",
        api_key=api_key,
        max_tokens=4096,
    )
    
    # Get ast-grep MCP tools via HTTP bridge
    tools = await get_ast_grep_tools(mcp_url, project_path)
    
    if not tools:
        raise RuntimeError("No ast-grep tools available - agent cannot function without tools!")
    
    print(f"[Analyzer Agent] Tools available: {[t.metadata.name for t in tools]}")
    
    # Create ReAct agent with tools (new LlamaIndex API)
    agent = ReActAgent(
        tools=tools,
        llm=llm,
        verbose=True,
    )
    
    print(f"[Analyzer Agent] Created ReActAgent with {len(tools)} ast-grep tools")
    return agent


async def analyze_with_agent(
    agent: ReActAgent,
    analysis_type: str,
    struct_name: str,
    struct_code: str,
    file_path: str,
    project_path: str,
) -> Dict[str, Any]:
    """
    Analyze a struct using the agent with ast-grep tools.
    
    The agent MUST use ast-grep tools to explore the code before
    providing classification and migration recommendations.
    """
    prompt = f"""REQUIRED: You MUST use the find_code tool before answering.

Analyze struct '{struct_name}' at {file_path}

Step 1: Use find_code tool to search for "pub struct {struct_name}" in {project_path}
Step 2: Use find_code tool to search for "EntityMeta" pattern usage in {project_path}
Step 3: Based on tool results, classify the struct

```rust
{struct_code}
```

After using tools, return JSON:
{{"classification": "domain_entity|dto|component", "should_migrate": bool, "priority": 1-5, "reasoning": "based on ast-grep findings", "ast_grep_queries": ["queries you ran"]}}"""

    try:
        # Create a context for this analysis
        from llama_index.core.workflow import Context
        ctx = Context(agent)
        result = await agent.run(user_msg=prompt, ctx=ctx)
        response_text = str(result.get("response", result) if isinstance(result, dict) else result)
        
        # Try to parse JSON from response
        try:
            json_start = response_text.find("{")
            json_end = response_text.rfind("}") + 1
            if json_start >= 0 and json_end > json_start:
                json_str = response_text[json_start:json_end]
                result = json.loads(json_str)
                # Ensure required fields
                result.setdefault("classification", None)
                result.setdefault("should_migrate", False)
                result.setdefault("priority", 0)
                result.setdefault("reasoning", "")
                result.setdefault("ast_grep_queries", [])
                result.setdefault("suggested_fix", None)
                result.setdefault("dependencies", [])
                result.setdefault("error", None)
                return result
        except json.JSONDecodeError:
            pass
        
        # Return raw response if JSON parsing fails
        return {
            "classification": None,
            "should_migrate": False,
            "priority": 0,
            "reasoning": response_text,
            "ast_grep_queries": [],
            "suggested_fix": None,
            "dependencies": [],
            "error": "Failed to parse structured JSON response",
        }
        
    except Exception as e:
        return {
            "classification": None,
            "should_migrate": False,
            "priority": 0,
            "reasoning": "",
            "ast_grep_queries": [],
            "suggested_fix": None,
            "dependencies": [],
            "error": str(e),
        }


# ---------------------------------------------------------------------------
# Windmill entrypoint
# ---------------------------------------------------------------------------

def main(
    analysis_type: str,
    code: str,
    name: str,
    file_path: str,
    project_path: str,
    ast_grep_mcp_url: str = DEFAULT_AST_GREP_MCP_URL,
    context: Optional[str] = None,
) -> Dict[str, Any]:
    """
    Windmill entrypoint for schema analysis with ast-grep MCP.
    
    This runs in the familiar-eng workspace, completely separate from 
    the Familiar product. All LLM access is through LlamaIndex only.
    
    Connects to ast-grep MCP HTTP bridge which spawns ast-grep as subprocess
    (same as Cursor does, but accessible via HTTP for Docker).
    
    The LLM MUST use ast-grep tools - analysis will fail if
    the MCP bridge is not running.
    
    Args:
        analysis_type: Type of analysis (entity_classification, generate_migration, etc.)
        code: The Rust code to analyze
        name: Name of the struct/type
        file_path: Path to the source file
        project_path: Root path of the project (for ast-grep queries)
        ast_grep_mcp_url: URL of the ast-grep MCP HTTP bridge
        context: Additional context (unused, for compatibility)
    
    Returns:
        Analysis result with classification, reasoning, and suggestions.
        Will return error if ast-grep MCP is unavailable.
    """
    if not LLAMAINDEX_AVAILABLE:
        return {
            "classification": None,
            "should_migrate": False,
            "priority": 0,
            "reasoning": "",
            "ast_grep_queries": [],
            "suggested_fix": None,
            "dependencies": [],
            "error": f"LlamaIndex not available: {IMPORT_ERROR}. "
                     f"Install: pip install llama-index-tools-mcp llama-index-llms-anthropic",
        }
    
    async def run_analysis():
        try:
            # Create agent with ast-grep MCP (via HTTP bridge)
            agent = await create_analyzer_agent(ast_grep_mcp_url, project_path)
            
            # Run analysis with ast-grep tools
            result = await analyze_with_agent(
                agent=agent,
                analysis_type=analysis_type,
                struct_name=name,
                struct_code=code,
                file_path=file_path,
                project_path=project_path,
            )
            
            return result
            
        except RuntimeError as e:
            # MCP or API key error - return structured error
            return {
                "classification": None,
                "should_migrate": False,
                "priority": 0,
                "reasoning": "",
                "ast_grep_queries": [],
                "suggested_fix": None,
                "dependencies": [],
                "error": str(e),
            }
    
    return asyncio.run(run_analysis())


if __name__ == "__main__":
    # Test locally - connects to ast-grep MCP HTTP bridge
    # Start the bridge first: uvx --with starlette --with uvicorn python ast_grep_http_server.py
    test_code = """pub struct FamiliarEntity {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub entity_type: FamiliarEntityType,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}"""
    
    result = main(
        analysis_type="entity_classification",
        code=test_code,
        name="FamiliarEntity",
        file_path="src/types/conversation/entity.rs",
        project_path="/Users/erictheiss/familiar/docs/v4",
        ast_grep_mcp_url="http://127.0.0.1:8000/mcp",
    )
    print(json.dumps(result, indent=2))

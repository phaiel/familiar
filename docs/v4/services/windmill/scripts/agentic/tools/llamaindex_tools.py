"""LlamaIndex Tool wrappers for Familiar Fates pipeline.

Uses Fates-themed tool names:
- decima_classify: Classification (Decima measures the thread)
- physics: Emotional/cognitive physics
- rag_search: Memory search

Schema-first: tool inputs/outputs align with familiar-schemas.

## Windmill Environment

Tool I/O types should be imported from `familiar_types` pip package
when available. Tools themselves don't require complex type imports
since they work with primitives (strings, dicts).
"""
from typing import Any, Dict, List, Optional
import uuid

try:
    from llama_index.core.tools import FunctionTool
    TOOLS_AVAILABLE = True
except ImportError:
    TOOLS_AVAILABLE = False


# ---------------------------------------------------------------------------
# Tool Descriptions (for LLM selection)
# ---------------------------------------------------------------------------

DECIMA_CLASSIFY_DESCRIPTION = """Decima classifies and categorizes user content into entity types.
Use this tool when:
- The user shares a memory, thought, or experience that needs to be categorized
- You need to determine the type of content (MOMENT, THREAD, INTENT, NOTE, TASK, etc.)
- The user asks to "classify", "categorize", or "tag" something
- The user is logging an experience or journaling

Input: The text content to classify
Output: Classification results with entity types, probabilities, and physics dimensions"""

PHYSICS_DESCRIPTION = """Compute emotional and cognitive physics for content.
Use this tool when:
- The user shares emotional content that needs dimensional analysis
- You need to understand the emotional valence, arousal, or significance of content
- The user asks about how something "feels" or its emotional weight

Input: The text content to analyze
Output: Physics dimensions (valence, arousal, significance, epistemic)"""

RAG_DESCRIPTION = """Search the user's memory and knowledge base.
Use this tool when:
- The user asks about something they mentioned before
- You need to recall past conversations or stored memories
- The user asks to "remember", "find", or "search" for something
- Context from previous interactions would help answer the question

Input: Search query
Output: Relevant results from the user's memory bank"""


# ---------------------------------------------------------------------------
# Tool Implementations (Fates-themed)
# ---------------------------------------------------------------------------

def decima_classify(text: str) -> Dict[str, Any]:
    """
    Decima: Classify and categorize user content into entity types.
    
    Named after Decima, the Fate who measures the thread of life.
    
    Args:
        text: The text content to classify
        
    Returns:
        Classification result matching production format:
        - summary: [{types, subject}]
        - classified: count
    """
    # TODO: Wire to actual classifier service via HTTP call
    # For now, return schema-compliant stub data
    
    text_lower = text.lower()
    
    # Determine entity types based on content (simple heuristics)
    classifications = []
    
    # Check for query/intent
    if any(q in text_lower for q in ["?", "what", "how", "why", "when", "where", "who", "can you", "do you", "are you", "alive"]):
        classifications.append({"type": "INTENT", "probability": 0.70})
        classifications.append({"type": "THREAD", "probability": 0.50})
    # Check for task
    elif any(word in text_lower for word in ["task", "todo", "need to", "should", "must"]):
        classifications.append({"type": "TASK", "probability": 0.85})
        classifications.append({"type": "MOMENT", "probability": 0.30})
    # Check for memory/moment
    elif any(word in text_lower for word in ["remember", "memory", "yesterday", "last week", "this morning"]):
        classifications.append({"type": "MOMENT", "probability": 0.82})
        classifications.append({"type": "THREAD", "probability": 0.35})
    # Check for note/idea
    elif any(word in text_lower for word in ["idea", "thinking", "wonder", "note"]):
        classifications.append({"type": "NOTE", "probability": 0.78})
        classifications.append({"type": "THREAD", "probability": 0.25})
    else:
        classifications.append({"type": "MOMENT", "probability": 0.65})
        classifications.append({"type": "THREAD", "probability": 0.40})
    
    # Build types string like "INTENT(70%), THREAD(50%)"
    types_str = ", ".join([f"{c['type']}({int(c['probability']*100)}%)" for c in classifications])
    
    return {
        # Production format
        "summary": [
            {
                "types": types_str,
                "subject": None,
            }
        ],
        "classified": 1,
        # Additional fields for weave_result aggregation
        "classifications": classifications,
        "intent": "classify",
    }


def physics_tool(text: str) -> Dict[str, Any]:
    """
    Compute emotional and cognitive physics for content.
    
    Args:
        text: The text content to analyze
        
    Returns:
        Physics dimensions matching production format:
        - valence: -1.0 to 1.0 (negative to positive)
        - arousal: 0.0 to 1.0 (calm to excited)
        - significance: 0.0 to 1.0 (mass/importance)
        - epistemic: 0.0 to 1.0 (how much new information)
    """
    # TODO: Wire to actual physics service via HTTP call
    # For now, return schema-compliant stub with simple sentiment heuristics
    
    text_lower = text.lower()
    
    # Simple heuristics for physics values
    valence = 0.0
    if any(word in text_lower for word in ["happy", "great", "wonderful", "love", "excited", "alive"]):
        valence = 0.6
    elif any(word in text_lower for word in ["sad", "angry", "frustrated", "hate", "terrible"]):
        valence = -0.5
    
    arousal = 0.4
    if any(word in text_lower for word in ["excited", "urgent", "amazing", "incredible", "!"]):
        arousal = 0.7
    elif any(word in text_lower for word in ["calm", "peaceful", "quiet", "relaxed"]):
        arousal = 0.2
    
    significance = 0.3 + min(0.5, len(text) / 500)  # Longer = more significant
    epistemic = 0.5  # How much new information
    if "?" in text:
        epistemic = 0.7  # Questions have higher epistemic value
    
    return {
        "valence": valence,
        "arousal": arousal,
        "significance": significance,
        "epistemic": epistemic,
    }


def rag_search(query: str) -> Dict[str, Any]:
    """
    Search the user's memory and knowledge base.
    
    Args:
        query: Search query to find relevant memories
        
    Returns:
        Search results with relevant memories
    """
    # TODO: Wire to actual RAG/vector search service via HTTP call
    # For now, return placeholder
    
    result_id = str(uuid.uuid4())[:8]
    
    return {
        "query": query,
        "results": [
            {
                "id": f"mem-{result_id}",
                "content": f"Relevant memory for: {query[:50]}",
                "entity_type": "MOMENT",
                "score": 0.85,
            },
        ],
        "total_count": 1,
    }


def build_llamaindex_tools() -> List[Any]:
    """
    Build LlamaIndex FunctionTool wrappers for Fates pipeline.
    
    Tools:
    - decima_classify: Classification (Decima measures the thread)
    - physics: Emotional/cognitive physics
    - rag_search: Memory search
    """
    if not TOOLS_AVAILABLE:
        return []
    
    return [
        FunctionTool.from_defaults(
            fn=decima_classify, 
            name="decima_classify",
            description=DECIMA_CLASSIFY_DESCRIPTION,
        ),
        FunctionTool.from_defaults(
            fn=physics_tool, 
            name="physics",
            description=PHYSICS_DESCRIPTION,
        ),
        FunctionTool.from_defaults(
            fn=rag_search,             name="rag_search",
            description=RAG_DESCRIPTION,
        ),
    ]


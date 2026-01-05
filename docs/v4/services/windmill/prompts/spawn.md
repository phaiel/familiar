# Spawn Decision Prompts

Prompts for deciding which entities to create based on classification confidence.

## Decision Thresholds

```
SPAWN DECISION THRESHOLDS:

HIGH CONFIDENCE (≥ 0.8):
- Action: AUTO_SPAWN (create without user confirmation)
- Rationale: Strong enough signal to be confident
- Exception: BOND and FOCUS always require review

MEDIUM CONFIDENCE (0.5 - 0.8):
- Action: SUGGEST (recommend to user for review)
- Rationale: Reasonable signal but benefits from confirmation
- Show: Entity type, confidence, reason

LOW CONFIDENCE (0.3 - 0.5):
- Action: SKIP (log but don't create)
- Rationale: Too uncertain to warrant action
- May revisit if pattern emerges

VERY LOW CONFIDENCE (< 0.3):
- Action: CLARIFY (ask user for clarification)
- Rationale: Need more information
- Ask: "Would you like to create a [type] for this?"
```

## Spawn Suggestion Format

```
When suggesting entity spawn:

1. State the entity type and confidence:
   "[MOMENT] (85% confident)"

2. Explain the reason:
   "Action verb 'went' with past tense indicates a specific event"

3. Show the content that will be stored:
   "I went to the dog park this morning"

4. List any enrichments applied:
   - Primary thread: user
   - Temporal: "this morning"
   - Related threads: dog park (place)

5. If PULSE, show enriched content:
   Original: "it was so nice!"
   Enriched: "it was so nice going to the dog park this morning"

6. Request confirmation for SUGGEST actions:
   "Create this moment? [Yes/No/Edit]"
```

## Multiple Entity Handling

```
When multiple entity types detected:

1. Process in order of probability (highest first)
2. Maximum 3 entities per segment
3. Check for redundancy:
   - Don't create both MOMENT and PULSE for same action
   - Prefer more specific type

EXAMPLE:
Content: "I went to the park and felt so happy"
- MOMENT (0.9): "I went to the park" → AUTO_SPAWN
- PULSE (0.85): "felt so happy at the park" → AUTO_SPAWN (enriched)
- INTENT (0.2): Skip (too low)
```

## Manual-Only Types

```
Some entity types should NEVER auto-spawn:

BOND:
- Relationships are sensitive
- Need user confirmation of parties
- Could misinterpret casual mentions

FOCUS:
- Active goals are high-commitment
- User should consciously choose priorities
- Affects system behavior

Always suggest these with:
- Clear explanation of what will be created
- Option to edit before creating
- Warning about implications
```

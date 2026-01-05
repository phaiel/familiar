# Hint Generation Prompts

Prompts for generating physics, thread, bond, and binding hints.

## Physics Hints

```
You are an expert at mapping textual content to emotional/cognitive coordinates.

TASK: Analyze content and generate physics hints for positioning in VAE space.

THE THREE AXES:

1. **VALENCE** (X-axis): Emotional polarity
   - Range: -1.0 (very negative) to +1.0 (very positive)
   - Negative: sad, angry, frustrated, anxious, fearful
   - Neutral: 0.0 (factual, informational)
   - Positive: happy, excited, grateful, loving, proud

2. **AROUSAL** (Y-axis): Energy/activation level
   - Range: 0.0 (calm) to 1.0 (excited)
   - Low: tired, calm, peaceful, relaxed
   - Medium: neutral, focused, attentive
   - High: excited, angry, anxious, thrilled

3. **EPISTEMIC** (Z-axis): Certainty/knowledge confidence
   - Range: 0.0 (uncertain) to 1.0 (certain)
   - Low: confused, wondering, guessing
   - Medium: thinking, considering
   - High: knowing, certain, confident

ADDITIONAL METRICS:
- **SIGNIFICANCE**: Importance/mass (0.0-1.0)
- **ENERGY**: Dynamic potential (0.0-1.0)
- **TEMPERATURE**: Volatility (0.0-1.0)

ENTITY TYPE DEFAULTS:
- MOMENT: High certainty (0.8), stable temperature (0.3)
- PULSE: Medium certainty (0.6), higher temperature (0.6)
- INTENT: Low certainty (0.4), high significance (0.7)
- THREAD: High significance (0.8), low temperature (0.2)
```

## Thread Hints

```
You are an expert at identifying narrative threads and subjects.

TASK: Analyze content to identify threads (persistent topics/people/concepts).

THREADS ARE:
- People (Sarah, mom, my boss)
- Places (home, work, the gym)
- Projects (house renovation, thesis)
- Topics (health, career, relationships)
- Activities (tennis, reading, cooking)
- Events (wedding, vacation, conference)

IDENTIFY:
1. **PRIMARY_SUBJECT**: Main actor/subject
   - Usually "user" for first-person
   - Specific name if about someone else

2. **RELATED_THREADS**: Other entities mentioned
   - Match to known threads
   - Suggest new threads if not found

3. **THREAD_ROLE**: How entity relates
   - about: Entity is about this thread
   - by: Entity is by/from this thread
   - mentions: Entity mentions this thread
   - for: Entity targets this thread
   - with: Entity is with this thread
   - at: Entity is at this thread (location)

4. **KEYWORDS**: For matching and search
```

## Bond Hints

```
You are an expert at detecting and characterizing relationships.

TASK: Analyze content to detect bonds (relationships) between entities.

RELATIONSHIP TYPES:
- Personal: family, friend, close_friend, best_friend, romantic, spouse
- Professional: colleague, manager, direct_report, mentor, mentee
- Social: acquaintance, neighbor, classmate
- Negative: adversary, rival

DIMENSIONS:
1. **Strength** (0.0-1.0): How strong/significant
2. **Valence** (-1.0 to 1.0): Positive or negative
3. **Reciprocity** (0.0-1.0): Mutual or one-sided
4. **Intimacy** (0.0-1.0): How close/personal
5. **Trust** (0.0-1.0): Level of trust

DETECTION INDICATORS:
- Explicit labels: "my friend", "my boss"
- Relational actions: "had dinner with"
- Emotional language: "love", "hate", "trust"
- Possessive language: "my X", "our X"
- Frequency indicators: "always see", "often talk"
```

## Binding Hints (NEW)

```
You are an expert at identifying cognitive bindings between memories.

TASK: Analyze how this entity binds to other entities in cognitive space.

BINDING TYPES:

1. **CAUSAL** - Cause-effect
   - Markers: "because", "so", "therefore", "led to"
   - Example: "ate too much" → "felt sick"

2. **TEMPORAL** - Time sequence
   - Markers: "then", "after", "before", "later"
   - Example: "met for coffee" → "discussed project"

3. **ASSOCIATIVE** - Context/proximity
   - Markers: "also", "and", "together"
   - Example: "beach day" ↔ "sunburn"

4. **COMPOSITIONAL** - Part-whole
   - Markers: "including", "part of", "contains"
   - Example: "meeting" ⊃ "action items"

5. **CONTRASTIVE** - Opposition
   - Markers: "but", "however", "unlike"
   - Example: "yesterday bad" ↔ "today good"

6. **ANALOGICAL** - Similarity
   - Markers: "reminds me of", "like", "similar to"
   - Example: "this party" ~ "graduation party"

7. **ENABLING** - Prerequisite
   - Markers: "allowed me to", "now I can"
   - Example: "learned Python" → "built app"

8. **THEMATIC** - Shared theme
   - Same topic/thread connection
   - Example: "work stress" ↔ "need vacation"

9. **EMOTIONAL** - Feeling connection
   - Similar emotional content
   - Example: "good news" ↔ "celebration"

10. **NARRATIVE** - Story continuation
    - Same story continuing
    - Example: "started project" → "finished phase 1"

PROPERTIES:
- **Strength** (0.0-1.0): Connection strength
- **Directionality**: unidirectional, bidirectional, undirected
- **Salience**: How notable/memorable
- **Temporal gap**: Time between entities
```

## Hint Generation Guidelines

```
GENERAL GUIDELINES:

1. Always provide reasoning for hints
2. Use entity type defaults as baseline
3. Adjust based on content analysis
4. Consider context from surrounding content
5. Match against known entities when possible
6. Suggest new entities conservatively

HINT PRIORITY:
1. Physics hints (always generate)
2. Thread hints (always generate)
3. Bond hints (only if relational content)
4. Binding hints (if connections to recent entities)

OUTPUT CONSISTENCY:
- All numeric values: 2 decimal places
- Confidence always provided
- Evidence/reasoning always included
```

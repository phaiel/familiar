# Windmill Integration

Windmill provides workflow orchestration for familiar's:
- **Onboarding flows** - User signup, family creation, invitation acceptance
- **Agentic pipeline** - Multi-agent orchestration with LlamaIndex

## Quick Start

```bash
# 1. Start Windmill
cd services/windmill/
docker-compose up -d

# 2. Wait for Windmill to be ready (check http://localhost:8001)

# 3. Run setup script
./setup.sh

# 4. Copy the token to your .env file
echo "WINDMILL_TOKEN=<token from setup>" >> ../../.env

# 5. Start familiar-api
cd ../familiar-api
cargo run
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    familiar-core (Schema)                        │
│  src/types/          - Rust type definitions (single source)    │
│  generated/          - Auto-generated TypeScript + Python       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         WINDMILL                                │
│                                                                 │
│  flows/                    - JSON flow definitions (DAGs)       │
│    ├── onboarding_signup.flow.json                             │
│    └── onboarding_create_family.flow.json                      │
│                                                                 │
│  scripts/                  - Python-first scripts               │
│    ├── agentic/           - LlamaIndex multi-agent tools       │
│    └── concierge/         - Orchestrator and specialized tools │
│                                                                 │
│  prompts/                  - LLM prompt templates               │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      familiar-api (Consumer)                     │
│  WindmillClient           - Triggers flows, polls for results   │
│  run_onboarding_signup()  - Executes signup flow               │
│  run_onboarding_create_family() - Executes family creation     │
└─────────────────────────────────────────────────────────────────┘
```

## Directory Structure

```
services/windmill/
├── flows/                    # Workflow DAG definitions
│   ├── onboarding_signup.flow.json
│   └── onboarding_create_family.flow.json
│
├── scripts/                  # Python execution scripts
│   ├── agentic/             # LlamaIndex multi-agent
│   │   ├── llamaindex_adapter.py
│   │   ├── llamaindex_runner.py
│   │   └── tools/
│   │       └── llamaindex_tools.py
│   │
│   └── concierge/           # Specialized agent tools
│       ├── orchestrator.py
│       ├── classifier_tool.py
│       ├── physics_tool.py
│       └── schema.py
│
├── prompts/                  # LLM prompt templates
│   └── *.txt
│
├── docker-compose.yaml       # Windmill services
└── setup.sh                  # Initial setup script
```

## Onboarding Flows

### Signup Flow (`onboarding_signup.flow.json`)

Handles user registration:
1. Validate input (email format, password strength)
2. Check if user exists
3. Create user record
4. Record GDPR consents
5. Create session
6. Handle invitation (if provided)
7. Publish domain events

**Input:**
```typescript
interface SignupFlowInput {
  email: string;
  password?: string;
  name: string;
  invite_code?: string;
  consents: { terms: boolean; privacy: boolean };
  request_id: string;
}
```

**Output:**
```typescript
interface SignupFlowOutput {
  user_id: string;
  email: string;
  name: string;
  session_id: string;
  session_token: string;
  session_expires_at: string;
  needs_family: boolean;
  primary_tenant_id?: string;
}
```

### Create Family Flow (`onboarding_create_family.flow.json`)

Handles family (tenant) creation:
1. Validate input
2. Create tenant
3. Add user as admin member
4. Create personal channel
5. Create family channel
6. Publish domain events

**Input:**
```typescript
interface CreateFamilyFlowInput {
  user_id: string;
  family_name: string;
  user_name: string;
  user_email?: string;
  request_id: string;
}
```

**Output:**
```typescript
interface CreateFamilyFlowOutput {
  tenant_id: string;
  tenant_name: string;
  personal_channel_id: string;
  personal_channel_name: string;
  family_channel_id: string;
  family_channel_name: string;
}
```

## Schema-First Development

All types are defined in `familiar-core/src/types/` and auto-generated:

1. **Rust** (source of truth): `familiar_core::types::onboarding`
2. **TypeScript**: `familiar-core/generated/typescript/`
3. **Python**: `familiar-core/generated/python/`

Windmill Python scripts import from the generated Python types:

```python
from generated.api import SignupFlowInput, SignupFlowOutput
```

## Environment Variables

Required in Windmill:
- `DATABASE_URL` - PostgreSQL connection string
- `ANTHROPIC_API_KEY` - For LLM calls (if using Claude)
- `OPENAI_API_KEY` - For LLM calls (if using OpenAI)

Set via Windmill Variables UI or API.

## Development

### Adding a New Flow

1. Define types in `familiar-core/src/types/`
2. Run `cargo test --lib` to generate TypeScript
3. Create flow JSON in `flows/`
4. Add Windmill client method in `familiar-api`
5. Push to Windmill via API or CLI

### Testing Flows

```bash
# Via Windmill UI
# Navigate to http://localhost:8001 > Flows > Run

# Via API
curl -X POST http://localhost:8001/api/w/familiar/jobs/run/f/onboarding/signup \
  -H "Authorization: Bearer $WINDMILL_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "name": "Test User", ...}'
```

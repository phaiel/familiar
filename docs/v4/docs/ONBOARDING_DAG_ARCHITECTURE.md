# Onboarding DAG Architecture

## Overview

This document describes the event-driven onboarding architecture using Windmill DAGs, structured for future Redpanda integration.

## Current vs Target Architecture

### Current (Synchronous REST)
```
┌─────────┐     ┌─────────────┐     ┌──────────────┐
│   UI    │────▶│ familiar-api│────▶│   Database   │
│         │◀────│  (inline)   │◀────│ (TimescaleDB)│
└─────────┘     └─────────────┘     └──────────────┘
```
- All logic in route handlers
- Synchronous request/response
- No orchestration visibility

### Target (DAG + Async Pattern)
```
┌─────────┐     ┌─────────────┐     ┌──────────────┐
│   UI    │────▶│ familiar-api│────▶│  Windmill    │
│         │     │ (thin layer)│     │  (DAG owner) │
└────┬────┘     └──────┬──────┘     └──────┬───────┘
     │                 │                    │
     │    ┌────────────▼────────────┐      │
     │    │    async_tasks table    │◀─────┘
     │    │  (job tracking/state)   │
     │    └────────────┬────────────┘
     │                 │
     └─────────────────┴─────▶ Database (writes from Windmill)

Future: Replace direct Windmill trigger with Redpanda:
  API → Redpanda topic → Windmill subscribes
```

## Onboarding State Machine

```
                    ┌─────────────────────────────────────────┐
                    │           ONBOARDING STATES             │
                    └─────────────────────────────────────────┘
                    
    ┌──────────────┐
    │   INITIAL    │
    └──────┬───────┘
           │
           ▼
    ┌──────────────┐     ┌──────────────┐
    │  EMAIL_SENT  │────▶│ MAGIC_LINK   │ (if magic link flow)
    │              │     │  _CLICKED    │
    └──────┬───────┘     └──────┬───────┘
           │                    │
           ▼                    ▼
    ┌──────────────┐     ┌──────────────┐
    │   SIGNUP     │◀────│  (merge)     │
    │  _COMPLETE   │     │              │
    └──────┬───────┘     └──────────────┘
           │
           ├─────────────────┬──────────────────┐
           ▼                 ▼                  ▼
    ┌──────────────┐  ┌──────────────┐   ┌──────────────┐
    │  CREATE      │  │    JOIN      │   │    JOIN      │
    │  _FAMILY     │  │  _VIA_CODE   │   │  _VIA_REQUEST│
    └──────┬───────┘  └──────┬───────┘   └──────┬───────┘
           │                 │                  │
           │                 │           ┌──────▼───────┐
           │                 │           │   PENDING    │
           │                 │           │  _APPROVAL   │
           │                 │           └──────┬───────┘
           │                 │                  │
           ▼                 ▼                  ▼
    ┌──────────────────────────────────────────────────┐
    │                  ONBOARDING_COMPLETE             │
    └──────────────────────────────────────────────────┘
```

## Windmill Flows

### 1. Signup Flow (`f/familiar/onboarding/signup`)

**Input:**
```typescript
interface SignupInput {
  email: string;
  password?: string;      // null for magic-link
  name: string;
  invite_code?: string;   // if joining existing family
  consents: {
    terms: boolean;
    privacy: boolean;
  };
  request_id: string;
  ip_address?: string;
  user_agent?: string;
}
```

**DAG Steps:**
1. `validate_input` - Check email format, password strength
2. `check_existing_user` - Return early if email exists
3. `hash_password` - If password provided (skip for magic-link)
4. `create_user` - Insert into users table
5. `record_consents` - Insert consent records
6. `create_session` - Generate session token
7. `process_invite` - If invite_code, validate and add to family
8. `emit_event` - Publish `user.signup` event
9. `audit_log` - Record action

**Output:**
```typescript
interface SignupOutput {
  user_id: string;
  session_token: string;
  needs_family: boolean;
  joined_family_id?: string;
}
```

### 2. Magic Link Flow (`f/familiar/onboarding/magic_link`)

**Input:**
```typescript
interface MagicLinkInput {
  action: "request" | "consume";
  email?: string;         // for request
  token?: string;         // for consume
  invite_code?: string;
  request_id: string;
}
```

**DAG Steps (request):**
1. `check_user_exists` - Determine if login or signup
2. `generate_token` - Create secure token
3. `store_magic_link` - Save hashed token with metadata
4. `send_email` - Trigger email service (or return dev token)

**DAG Steps (consume):**
1. `validate_token` - Check hash, expiry, used status
2. `mark_used` - Update magic_links.used_at
3. `branch` - If user exists → login, else → create user
4. `create_session` - Generate session token
5. `emit_event` - Publish `user.magic_link_used` event

### 3. Create Family Flow (`f/familiar/onboarding/create_family`)

**Input:**
```typescript
interface CreateFamilyInput {
  user_id: string;
  family_name: string;
  request_id: string;
}
```

**DAG Steps:**
1. `validate_user` - Ensure user exists and needs family
2. `create_tenant` - Insert tenant record
3. `add_admin_member` - Insert tenant_member as admin
4. `update_user_primary` - Set user.primary_tenant_id
5. `create_family_channel` - Insert family chat channel
6. `create_nona_welcome` - Optional: Create welcome message from Nona
7. `emit_event` - Publish `family.created` event
8. `audit_log` - Record action

**Output:**
```typescript
interface CreateFamilyOutput {
  tenant_id: string;
  channel_id: string;
  tenant_name: string;
}
```

### 4. Accept Invitation Flow (`f/familiar/onboarding/accept_invitation`)

**Input:**
```typescript
interface AcceptInvitationInput {
  user_id: string;
  invite_code: string;
  request_id: string;
}
```

**DAG Steps:**
1. `get_invitation` - Fetch by code
2. `validate_invitation` - Check expiry, usage limits
3. `add_member` - Insert tenant_member
4. `increment_usage` - Update invitation use_count
5. `create_personal_channel` - Create user's personal channel
6. `emit_event` - Publish `family.member_joined` event
7. `audit_log` - Record action

## Async Task Table

For tracking async jobs and supporting the event-driven pattern:

```sql
-- Migration: 004_async_tasks.sql

CREATE TABLE async_tasks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Task identification
    task_type TEXT NOT NULL,  -- 'onboarding.signup', 'onboarding.create_family', etc.
    correlation_id TEXT NOT NULL,  -- Request ID for tracing
    
    -- Windmill job reference
    windmill_job_id TEXT,
    windmill_flow_path TEXT,
    
    -- State
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN (
        'pending',     -- Created, not yet sent to Windmill
        'queued',      -- Sent to Windmill, waiting to run
        'running',     -- Windmill job executing
        'completed',   -- Success
        'failed',      -- Error
        'cancelled'    -- Manually cancelled
    )),
    
    -- Input/Output (JSONB for flexibility)
    input JSONB NOT NULL,
    output JSONB,
    error_message TEXT,
    
    -- Actor
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    tenant_id UUID REFERENCES tenants(id) ON DELETE SET NULL,
    
    -- Timing
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    
    -- Retry tracking
    attempt_count INT NOT NULL DEFAULT 0,
    max_attempts INT NOT NULL DEFAULT 3,
    next_retry_at TIMESTAMPTZ
);

-- Indexes for common queries
CREATE INDEX idx_async_tasks_status ON async_tasks(status);
CREATE INDEX idx_async_tasks_correlation ON async_tasks(correlation_id);
CREATE INDEX idx_async_tasks_user ON async_tasks(user_id);
CREATE INDEX idx_async_tasks_type_status ON async_tasks(task_type, status);
CREATE INDEX idx_async_tasks_pending_retry ON async_tasks(next_retry_at) 
    WHERE status IN ('pending', 'failed');

-- Convert to hypertable for time-series queries
SELECT create_hypertable('async_tasks', 'created_at', if_not_exists => TRUE);
```

## API Pattern (Thin Layer)

The API becomes a thin layer that:
1. Validates authentication
2. Creates async_task record
3. Triggers Windmill (or publishes to Redpanda)
4. Returns task reference for polling

```rust
// Example: POST /api/auth/signup (async version)
pub async fn signup_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SignupRequest>,
) -> impl IntoResponse {
    // 1. Basic validation only
    if !req.accept_terms || !req.accept_privacy {
        return (StatusCode::BAD_REQUEST, Json(ErrorResponse { ... }));
    }

    // 2. Create async task
    let task = AsyncTask {
        task_type: "onboarding.signup".to_string(),
        correlation_id: Uuid::new_v4().to_string(),
        input: serde_json::to_value(&req).unwrap(),
        user_id: None,  // Not yet created
        ..Default::default()
    };
    let task_id = store.create_async_task(&task).await?;

    // 3. Trigger Windmill flow
    let windmill_input = SignupInput {
        email: req.email,
        password: Some(req.password),
        name: req.name,
        invite_code: req.invite_code,
        consents: Consents { terms: true, privacy: true },
        request_id: task.correlation_id.clone(),
        ip_address: extract_ip(&headers),
        user_agent: extract_user_agent(&headers),
    };
    
    let job = windmill.trigger_flow("f/familiar/onboarding/signup", &windmill_input).await?;
    
    // 4. Update task with job reference
    store.update_task_job(task_id, &job.job_id, "f/familiar/onboarding/signup").await?;

    // 5. Return task reference (UI can poll for completion)
    Json(AsyncTaskResponse {
        task_id,
        status: "queued",
        poll_url: format!("/api/tasks/{}", task_id),
    })
}
```

## UI Polling Pattern

```typescript
// hooks/useAsyncTask.ts
export function useAsyncTask(taskId: string | null) {
  const [task, setTask] = useState<AsyncTask | null>(null);
  const [isPolling, setIsPolling] = useState(false);

  useEffect(() => {
    if (!taskId) return;
    
    setIsPolling(true);
    const poll = async () => {
      const res = await fetch(`/api/tasks/${taskId}`);
      const data = await res.json();
      setTask(data);
      
      if (data.status === 'completed' || data.status === 'failed') {
        setIsPolling(false);
        return;
      }
      
      // Continue polling
      setTimeout(poll, 1000);
    };
    
    poll();
    
    return () => setIsPolling(false);
  }, [taskId]);

  return { task, isPolling };
}
```

## Redpanda Transition Path

When ready to move to Redpanda:

1. **Topics:**
   - `onboarding.commands` - Inbound (API publishes)
   - `onboarding.events` - Outbound (Windmill publishes)

2. **API Change:**
   ```rust
   // Instead of direct Windmill trigger:
   // windmill.trigger_flow("f/familiar/onboarding/signup", &input).await?;
   
   // Publish to Redpanda:
   redpanda.produce("onboarding.commands", OnboardingCommand::Signup(input)).await?;
   ```

3. **Windmill Subscription:**
   - Use Windmill's Kafka connector or a polling script
   - Consume from `onboarding.commands`
   - Execute appropriate flow
   - Publish result to `onboarding.events`

4. **Benefits:**
   - Decoupled: API doesn't know about Windmill
   - Replayable: Events stored in Redpanda
   - Scalable: Multiple Windmill workers can consume
   - Resilient: Redpanda handles backpressure

## Event Types

```typescript
// Domain events published by Windmill flows

type OnboardingEvent = 
  | { type: "user.signup.started"; user_id: string; email: string }
  | { type: "user.signup.completed"; user_id: string; needs_family: boolean }
  | { type: "user.magic_link.requested"; email: string }
  | { type: "user.magic_link.consumed"; user_id: string; is_new_user: boolean }
  | { type: "family.created"; tenant_id: string; admin_user_id: string }
  | { type: "family.member.joined"; tenant_id: string; user_id: string; via: "invite" | "request" }
  | { type: "family.member.request.submitted"; tenant_id: string; user_id: string }
  | { type: "family.member.request.reviewed"; tenant_id: string; user_id: string; approved: boolean };
```

## Redpanda Transition Guide

### Phase 1: Prepare (Current State)
The current architecture already supports the async pattern:
- **API** creates `async_tasks` records
- **API** triggers Windmill flows directly
- **UI** polls `/api/tasks/:id` for completion

### Phase 2: Add Redpanda (Intermediate)
Run Redpanda alongside direct Windmill triggers:

```yaml
# docker-compose.redpanda.yml
services:
  redpanda:
    image: docker.redpanda.com/redpandadata/redpanda:latest
    command:
      - redpanda start
      - --smp 1
      - --memory 1G
      - --overprovisioned
      - --kafka-addr internal://0.0.0.0:9092,external://0.0.0.0:19092
      - --advertise-kafka-addr internal://redpanda:9092,external://localhost:19092
    ports:
      - "19092:19092"
      - "9644:9644"  # Admin API
```

**Topics to Create:**
```bash
rpk topic create onboarding.commands --partitions 3
rpk topic create onboarding.events --partitions 3
rpk topic create onboarding.dlq --partitions 1  # Dead letter queue
```

### Phase 3: Dual-Write Pattern
Publish to both Windmill and Redpanda during transition:

```rust
// API route during transition
pub async fn signup_handler(...) -> impl IntoResponse {
    // 1. Create async_task record (unchanged)
    let task = create_async_task(&store, ...).await?;
    
    // 2. Dual-write: Windmill + Redpanda
    let input = SignupFlowInput { ... };
    
    // Direct Windmill trigger (existing)
    if let Some(windmill) = &state.windmill {
        windmill.trigger_flow("f/familiar/onboarding/signup", &input).await?;
    }
    
    // Also publish to Redpanda (new)
    if let Some(producer) = &state.redpanda {
        let command = OnboardingCommand::Signup(input);
        producer.send("onboarding.commands", &command).await?;
    }
    
    // 3. Return task reference (unchanged)
    Json(AsyncTaskCreated { task_id, ... })
}
```

### Phase 4: Windmill Consumer
Create a Windmill script that consumes from Redpanda:

```python
# scripts/onboarding/redpanda_consumer.py
from kafka import KafkaConsumer
import json
import wmill

def main():
    consumer = KafkaConsumer(
        'onboarding.commands',
        bootstrap_servers=wmill.get_variable('u/admin/REDPANDA_BROKERS'),
        group_id='windmill-onboarding',
        auto_offset_reset='earliest',
        value_deserializer=lambda m: json.loads(m.decode('utf-8'))
    )
    
    for message in consumer:
        command = message.value
        correlation_id = command.get('request_id')
        
        try:
            if command['type'] == 'signup':
                result = wmill.run_flow('f/familiar/onboarding/signup', command['payload'])
            elif command['type'] == 'create_family':
                result = wmill.run_flow('f/familiar/onboarding/create_family', command['payload'])
            # ... etc
            
            # Publish success event
            publish_event('onboarding.events', {
                'type': f"{command['type']}.completed",
                'correlation_id': correlation_id,
                'result': result
            })
        except Exception as e:
            # Publish to DLQ
            publish_event('onboarding.dlq', {
                'original': command,
                'error': str(e),
                'correlation_id': correlation_id
            })
```

### Phase 5: Remove Direct Windmill Triggers
Once Redpanda consumer is stable:

```rust
// API route after transition
pub async fn signup_handler(...) -> impl IntoResponse {
    let task = create_async_task(&store, ...).await?;
    
    // Only publish to Redpanda
    let command = OnboardingCommand::Signup(SignupFlowInput { ... });
    state.redpanda.send("onboarding.commands", &command).await?;
    
    Json(AsyncTaskCreated { task_id, ... })
}
```

### Phase 6: Event-Driven Updates
Update `async_tasks` table from event stream:

```python
# scripts/onboarding/event_processor.py
def main():
    consumer = KafkaConsumer('onboarding.events', ...)
    
    for message in consumer:
        event = message.value
        correlation_id = event.get('correlation_id')
        
        if event['type'].endswith('.completed'):
            update_task_status(correlation_id, 'completed', event['result'])
        elif event['type'].endswith('.failed'):
            update_task_status(correlation_id, 'failed', error=event['error'])
```

### Benefits After Migration

1. **Decoupling**: API doesn't know about Windmill
2. **Replayability**: All commands stored in Redpanda (configurable retention)
3. **Scalability**: Multiple Windmill workers can consume from same topic
4. **Resilience**: Redpanda handles backpressure, retries
5. **Observability**: Kafka tooling for monitoring, lag tracking
6. **Exactly-once**: Redpanda transactions + idempotent consumers

### Rollback Plan
Keep the dual-write flag for quick rollback:

```rust
let use_redpanda = std::env::var("USE_REDPANDA").is_ok();

if use_redpanda {
    state.redpanda.send(...).await?;
} else {
    state.windmill.trigger_flow(...).await?;
}
```

## Implementation Checklist

- [x] Create `004_async_tasks.sql` migration
- [x] Add Rust types for onboarding input/output (`types/onboarding.rs`)
- [x] Create Windmill scripts for each step (inline in flows)
- [x] Create Windmill flows (DAGs)
- [x] Update API routes to async pattern (`routes/async_tasks.rs`)
- [x] Add `/api/tasks/:id` endpoint for polling
- [ ] Update UI to use polling pattern (`useAsyncTask` hook)
- [ ] Add observability (OpenTelemetry tracing)
- [x] Document Redpanda migration steps (above)
- [ ] Create Redpanda docker-compose overlay
- [ ] Implement Redpanda consumer in Windmill


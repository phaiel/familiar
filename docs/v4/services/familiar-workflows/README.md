# familiar-workflows

TypeScript workflows for Temporal - the "brain" orchestrating the Rust "muscle".

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    familiar-workflows (TypeScript)               │
│                         "The Brain"                              │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  loomWorkflow                                            │    │
│  │    ├── FatesGate    ─────────┐                          │    │
│  │    ├── FatesMorta   ─────────┤                          │    │
│  │    ├── FatesDecima  ─────────┼──► gRPC to Rust Worker   │    │
│  │    └── FatesNona    ─────────┘                          │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Temporal Server                              │
│                      localhost:7233                              │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    familiar-daemon (Rust)                        │
│                        "The Muscle"                              │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  HotState (Arc<...>)                                    │    │
│  │    ├── ContractEnforcer (SIMD-JSON + compiled schemas)  │    │
│  │    └── SeaORM Connection Pool                           │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

## Quick Start

```bash
# Install dependencies
npm install

# Start the workflow worker (requires Temporal server running)
npm run start:worker

# In another terminal, test with the client
npm run start:client
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TEMPORAL_ADDRESS` | `localhost:7233` | Temporal server gRPC address |
| `TEMPORAL_NAMESPACE` | `default` | Temporal namespace |
| `TEMPORAL_TASK_QUEUE` | `fates-pipeline` | Task queue for workflows |

## Workflows

### loomWorkflow

Orchestrates the Fates pipeline stage-by-stage:

1. **Gate** - Classification and routing
2. **Morta** - Content segmentation
3. **Decima** - Entity extraction
4. **Nona** - Response generation

Each stage is a separate activity call, providing:
- Visual progress in Temporal UI
- Granular retries (if Decima fails, only Decima retries)
- Ability to insert approval steps between stages

### loomWorkflowFast

Calls the entire pipeline as a single activity. Less visibility but simpler.

## Development

```bash
# Build TypeScript
npm run build

# Watch mode
npm run build:watch

# Type check
npx tsc --noEmit
```

## Type Synchronization

Types in `src/types/` should match Rust structs in `familiar-core`.

To sync types from Rust:
```bash
cargo xtask sync-temporal
```

This copies ts-rs generated bindings to `src/types/`.






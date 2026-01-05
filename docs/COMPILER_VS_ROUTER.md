# Compiler vs Router: Clear Separation of Responsibilities

## Overview

The **Architecture Compiler** and **Runtime Router** have distinct, non-overlapping responsibilities in the Familiar platform. This document clarifies their roles to prevent confusion and ensure clean architecture.

## Architecture Compiler (familiar-schemas)

### Location
- **Crate**: `familiar-schemas/packages/familiar-schemas/`
- **Execution**: Build-time (xtask commands)
- **Input**: JSON schemas with logical definitions
- **Output**: Static artifacts and validation

### Responsibilities

#### ✅ DOES: Logical Analysis & Generation
- **Schema Validation**: Ensure schemas are syntactically correct and follow meta-schemas
- **CEL Compilation**: Validate CEL expressions can be parsed (syntax only, no execution)
- **Code Generation**: Generate Rust types, TypeScript interfaces, etc. from schemas
- **Graph Construction**: Build static dependency graphs from `$ref` relationships
- **Static Analysis**: Detect cycles, validate references, check compatibility

#### ✅ DOES: Infrastructure Template Generation
- **K8s Manifest Templates**: Generate deployment/service/HPA manifests from node resource requirements
- **Helm Charts**: Create parameterized Helm charts for different environments
- **Terraform Modules**: Generate infrastructure-as-code from node capacity definitions
- **Docker Compose**: Generate development environment configurations

#### ❌ DOES NOT: Runtime Decisions
- No live telemetry consumption
- No dynamic routing decisions
- No resource allocation at runtime
- No CEL execution with live data

### Example Commands
```bash
# Schema validation and analysis
cargo xtask validate-cel
cargo xtask graph

# Code generation
cargo run --bin rust-codegen

# Infrastructure generation (templates)
cargo xtask infra --env production
```

## Runtime Router (familiar-router)

### Location
- **Crate**: `services/familiar-router/`
- **Execution**: Runtime (in familiar-core/familiar-daemon)
- **Input**: Live telemetry + compiled schemas
- **Output**: Routing decisions + resource leases

### Responsibilities

#### ✅ DOES: Dynamic Runtime Decisions
- **CEL Execution**: Evaluate constraints and routing policies with live telemetry
- **Resource Leasing**: Track and allocate node resources to prevent OOM
- **Load Balancing**: Make intelligent routing decisions based on current system state
- **Telemetry Integration**: Consume live metrics from Prometheus/Kubernetes
- **Health Checking**: Continuously validate node health and capacity

#### ✅ DOES: Operational Intelligence
- **Routing Traces**: Record why messages were routed to specific nodes
- **Resource Tracking**: Monitor utilization and prevent over-subscription
- **Failure Recovery**: Handle node failures and re-route messages
- **Performance Monitoring**: Track routing latency and success rates

#### ❌ DOES NOT: Schema Analysis
- No schema parsing or validation
- No code generation
- No static analysis
- No infrastructure template generation

### Example Usage
```rust
// In familiar-core
let router = Router::new(telemetry_provider);

// Register node capacities (from schema-derived config)
router.register_node_capacity("familiar-daemon", capacity);

// Make routing decision with live data
let decision = router.route("kafka:weave", &input).await?;

// Track resources
router.release_resources(&decision.node_id, &decision.lease_id)?;
```

## Key Architectural Boundaries

### Data Flow Separation

```
Schemas (JSON) → Compiler → Static Artifacts (Rust code, K8s templates)
                      ↓
Telemetry (Live) → Router → Runtime Decisions (routing, leasing)
```

### Decision Criteria

| Concern | Compiler | Router |
|---------|----------|--------|
| **When** | Build-time | Runtime |
| **Data Source** | Static schemas | Live telemetry |
| **Output Type** | Code/templates | Decisions/leases |
| **Failure Impact** | Build failure | Runtime degradation |
| **Caching** | Pre-computed | Real-time |
| **Testing** | Unit tests | Integration tests |

### Communication Pattern

The Compiler and Router communicate through **compiled artifacts**:

1. **Compiler** generates resource capacity configurations from schemas
2. **Router** consumes these configurations at startup
3. **Router** makes decisions based on live telemetry + compiled constraints
4. **Router** records traces that can be correlated with schema versions

## Example: Large Message Routing

### Schema Definition (Compiler Domain)
```json
{
  "routing_policy": "input.content_length > 10000 ? 'high-memory-pool' : 'standard-pool'",
  "constraints": {
    "memory": "node.available_memory > 2000000"
  }
}
```

### Runtime Execution (Router Domain)
```rust
// Compiler output: pool mappings
let pools = HashMap::from([
    ("high-memory-pool", vec!["familiar-daemon"]),
    ("standard-pool", vec!["familiar-worker"]),
]);

// Router execution with live data
async fn route_large_message(router: &Router, input: &Value) -> RouterResult<RouteDecision> {
    let size = estimate_size(input);
    
    if size > 10000 {
        // Find node in high-memory-pool that satisfies constraints
        router.find_best_node("high-memory-pool", input).await
    } else {
        router.find_best_node("standard-pool", input).await
    }
}
```

## Benefits of Clear Separation

### **Maintainability**
- Compiler changes don't affect runtime performance
- Router changes don't require schema recompilation
- Independent testing and deployment

### **Performance**
- Compiler runs once at build time
- Router optimized for low-latency decisions
- Separate caching strategies for each

### **Reliability**
- Schema validation failures caught at build time
- Runtime routing failures degrade gracefully
- Independent failure domains

### **Evolvability**
- Schema language can evolve without affecting runtime
- Routing algorithms can improve without schema changes
- Technology stacks can differ (Rust vs whatever for compiler)

## Conclusion

**The Architecture Compiler creates the "rules of the game" - the Router enforces them with live data.**

This separation enables:
- **Static guarantees** (schema validation, type safety)
- **Dynamic adaptation** (load balancing, resource management)
- **Independent evolution** (compiler improvements don't affect runtime)

The result is a robust, scalable distributed system where **schemas define what should happen, and the router makes it happen optimally**.

# Familiar Infrastructure

This document describes the canonical infrastructure configuration for Familiar.

## Overview

All infrastructure services are managed via a single `docker-compose.yml` in the project root.

```
docker compose up -d      # Start all services
docker compose down       # Stop all services
docker compose down -v    # Stop and destroy all data (CAUTION!)
```

## Services

### Port Mappings (Development)

| Service | Port | Description |
|---------|------|-------------|
| TigerData (PostgreSQL) | 5433 | Main application database (TimescaleDB + pgvector) |
| Qdrant | 6333, 6334 | Vector database (REST, gRPC) |
| Windmill | 8000 | Workflow orchestration UI and API |
| MinIO | 9000, 9001 | Object storage (API, Console) |
| Redpanda (Kafka) | 19092 | Message broker |
| Redpanda Console | 8080 | Kafka management UI |
| Schema Registry | 18081 | Redpanda schema registry |
| familiar-api | 3001 | Rust backend API |
| familiar-ui | 3000 | Next.js frontend |

### Volume Names

| Volume | Purpose |
|--------|---------|
| `familiar-tigerdata` | Application database |
| `familiar-qdrant` | Vector embeddings |
| `familiar-windmill-db` | Windmill PostgreSQL (contains both workspaces) |
| `familiar-windmill-lsp` | Windmill LSP cache |
| `familiar-minio` | Media files |
| `familiar-redpanda` | Kafka data and schemas |

## Windmill Workspaces

A single Windmill instance serves multiple workspaces:

```
Windmill Instance (localhost:8000)
├── familiar (App user flows)
│   ├── f/onboarding/signup
│   ├── f/onboarding/create_family
│   ├── f/onboarding/accept_invitation
│   └── f/agentic/main (LlamaIndex/rig pipeline)
│
└── familiar-eng (Engineering tools)
    └── f/analyzer/schema (Schema compliance checker)
```

### Workspace Configuration

| Workspace | Purpose | Used By | Token Env Var |
|-----------|---------|---------|---------------|
| `familiar` | App user flows | familiar-api, familiar-worker | `WINDMILL_TOKEN` |
| `familiar-eng` | Schema analysis | familiar-core | `WINDMILL_TOKEN` |

### Flow Architecture

Both workspaces use the same LlamaIndex-based architecture today:

1. **familiar/agentic/main** - Main AI pipeline for weave processing
   - Gate (intent classification)
   - Morta (segmentation)
   - Decima (classification)
   - Nona (response generation)

2. **familiar/onboarding/*** - User onboarding DAGs
   - signup: Create user, provision defaults
   - create_family: Create tenant with family channel
   - accept_invitation: Join existing family

3. **familiar-eng/analyzer/schema** - Schema compliance validation
   - Uses ast-grep for pattern matching
   - LLM escalation for complex cases

## Environment Variables

### familiar-api/.env

```bash
# Database
DATABASE_URL=postgres://familiar:familiarpass@localhost:5433/familiar

# Windmill
WINDMILL_URL=http://localhost:8000
WINDMILL_WORKSPACE=familiar
WINDMILL_TOKEN=<your-token>
WINDMILL_AGENTIC_FLOW=f/agentic/main

# MinIO
MINIO_ENDPOINT=http://localhost:9000
MINIO_ACCESS_KEY=familiar
MINIO_SECRET_KEY=familiarpass
MINIO_BUCKET=familiar-media

# Kafka/Redpanda
KAFKA_BOOTSTRAP_SERVERS=localhost:19092
KAFKA_GROUP_ID=familiar-api-group
KAFKA_COMMANDS_TOPIC=course.commands
KAFKA_EVENTS_TOPIC=course.events
KAFKA_TRACE_TOPIC=course.trace

# API
PORT=3001
```

### familiar-worker/.env

```bash
# Kafka/Redpanda
KAFKA_BOOTSTRAP_SERVERS=localhost:19092
KAFKA_GROUP_ID=familiar-worker
KAFKA_COMMANDS_TOPIC=course.commands
KAFKA_EVENTS_TOPIC=course.events
KAFKA_TRACE_TOPIC=course.trace

# Windmill
WINDMILL_URL=http://localhost:8000
WINDMILL_WORKSPACE=familiar
WINDMILL_TOKEN=<your-token>
```

### familiar-core/.env

```bash
WINDMILL_URL=http://localhost:8000
WINDMILL_WORKSPACE=familiar-eng
WINDMILL_TOKEN=<your-token>
```

## Kafka Topics

| Topic | Purpose | Producer | Consumer |
|-------|---------|----------|----------|
| `course.commands` | User weave commands | familiar-api | familiar-worker |
| `course.events` | State change events | familiar-worker | (projectors) |
| `course.trace` | Processing trace events | familiar-worker | familiar-api (WebSocket) |

## Development vs Production

### Port Mapping

| Service | Dev Port | Prod Port | Notes |
|---------|----------|-----------|-------|
| TigerData | 5433 | 5432 | Standard PostgreSQL |
| Windmill | 8000 | 8000 | Behind reverse proxy |
| MinIO | 9000 | - | Use S3 in prod |
| Redpanda | 19092 | 9092 | Internal only in prod |
| familiar-api | 3001 | 3001 | Behind reverse proxy |
| familiar-ui | 3000 | 3000 | Behind reverse proxy |

### Production Considerations

1. **Windmill**: Deploy via Kubernetes or managed service
2. **MinIO**: Replace with AWS S3 or compatible
3. **Redpanda**: Use Redpanda Cloud or self-hosted cluster
4. **TigerData**: Use managed TimescaleDB or RDS
5. **Qdrant**: Use Qdrant Cloud or self-hosted cluster

## Troubleshooting

### Check Service Status

```bash
docker compose ps
docker compose logs <service>
```

### Reset Windmill

```bash
# Caution: This destroys all Windmill data
docker compose down -v
docker volume rm familiar-windmill-db
docker compose up -d
```

### Verify Windmill Flows

```bash
curl -s http://localhost:8000/api/w/familiar/flows/list \
  -H "Authorization: Bearer $WINDMILL_TOKEN" | jq '.[].path'
```

### Check Kafka Topics

```bash
docker exec familiar-redpanda rpk topic list --brokers localhost:9092
docker exec familiar-redpanda rpk topic consume course.commands --brokers localhost:9092 -n 1
```












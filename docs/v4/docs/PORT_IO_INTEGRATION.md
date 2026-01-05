# Port.io Integration for Familiar Schema Architecture

**Platform:** Port.io (getport.io)  
**Use Case:** Visualize dependencies, manage schema versions, scaffold services

---

## Why Port.io Is Perfect for This

Port.io advantages over Backstage:
- ✅ **Better out-of-the-box:** Less setup, more features immediately
- ✅ **Better dependency visualization:** Native graph view
- ✅ **Scorecards:** Track schema version compliance across services
- ✅ **Self-service actions:** Built-in scaffolding without custom code
- ✅ **No hosting required:** SaaS (or self-hosted if needed)
- ✅ **Better UI/UX:** Modern, intuitive interface

---

## Architecture Overview

```
Port.io Platform
├── Software Catalog
│   ├── Schema Library Blueprint
│   │   └── familiar-schemas (v0.1.0, v0.2.0, etc.)
│   ├── Service Blueprint
│   │   ├── motif-service (uses schemas v0.1.0)
│   │   ├── thread-service (uses schemas v0.1.0)
│   │   └── bond-service (uses schemas v0.0.9) ⚠️
│   └── Template Blueprint
│       ├── Microservice Template
│       ├── GraphQL Gateway Template
│       └── Event Processor Template
│
├── Self-Service Actions
│   ├── Generate Service from Schema
│   ├── Update Schema Version
│   └── Check Breaking Changes
│
├── Scorecards
│   ├── Schema Version Compliance
│   ├── Breaking Change Risk
│   └── Test Coverage
│
└── Dependency Graph
    └── Visual: schemas → services
```

---

## Setup Guide

### 1. Define Blueprints (Schema for Port.io Catalog)

#### Schema Library Blueprint

```json
{
  "identifier": "schemaLibrary",
  "title": "Schema Library",
  "icon": "Blueprint",
  "schema": {
    "properties": {
      "version": {
        "title": "Version",
        "type": "string",
        "description": "Semantic version (e.g., v0.1.0)"
      },
      "gitTag": {
        "title": "Git Tag",
        "type": "string",
        "format": "url"
      },
      "publishedAt": {
        "title": "Published At",
        "type": "string",
        "format": "date-time"
      },
      "breakingChanges": {
        "title": "Breaking Changes",
        "type": "array",
        "items": {
          "type": "string"
        }
      },
      "entities": {
        "title": "Entities",
        "type": "array",
        "description": "Available entity types",
        "items": {
          "type": "string",
          "enum": ["Motif", "Thread", "Bond", "Moment", "Intent", "Focus", "Filament"]
        }
      },
      "components": {
        "title": "Components",
        "type": "array",
        "items": {
          "type": "string"
        }
      }
    },
    "required": ["version", "gitTag"]
  },
  "calculationProperties": {},
  "relations": {
    "consumedBy": {
      "title": "Consumed By",
      "target": "service",
      "many": true
    }
  }
}
```

#### Service Blueprint

```json
{
  "identifier": "service",
  "title": "Service",
  "icon": "Microservice",
  "schema": {
    "properties": {
      "language": {
        "title": "Language",
        "type": "string",
        "enum": ["rust", "python", "typescript"]
      },
      "schemaVersion": {
        "title": "Schema Version",
        "type": "string",
        "description": "Version of familiar-schemas used"
      },
      "entities": {
        "title": "Entities Used",
        "type": "array",
        "items": {
          "type": "string"
        }
      },
      "lastDeployed": {
        "title": "Last Deployed",
        "type": "string",
        "format": "date-time"
      },
      "repository": {
        "title": "Repository",
        "type": "string",
        "format": "url"
      }
    },
    "required": ["language", "schemaVersion"]
  },
  "calculationProperties": {
    "schemaUpToDate": {
      "title": "Schema Up To Date",
      "type": "boolean",
      "calculation": ".properties.schemaVersion == .relations.schemas.properties.version | .[0]"
    }
  },
  "relations": {
    "schemas": {
      "title": "Uses Schema Version",
      "target": "schemaLibrary",
      "many": false,
      "required": true
    }
  }
}
```

#### Template Blueprint

```json
{
  "identifier": "template",
  "title": "Service Template",
  "icon": "Template",
  "schema": {
    "properties": {
      "templateType": {
        "title": "Template Type",
        "type": "string",
        "enum": ["microservice", "graphql-gateway", "event-processor"]
      },
      "supportsEntities": {
        "title": "Supported Entities",
        "type": "array",
        "items": {
          "type": "string"
        }
      },
      "repository": {
        "title": "Template Repository",
        "type": "string",
        "format": "url"
      }
    }
  }
}
```

---

### 2. Populate Catalog (via API or GitHub Integration)

#### Ingest Schema Versions

```python
# scripts/sync_to_port.py
import requests
import subprocess

PORT_CLIENT_ID = os.getenv("PORT_CLIENT_ID")
PORT_CLIENT_SECRET = os.getenv("PORT_CLIENT_SECRET")

def get_port_token():
    response = requests.post(
        "https://api.getport.io/v1/auth/access_token",
        json={
            "clientId": PORT_CLIENT_ID,
            "clientSecret": PORT_CLIENT_SECRET
        }
    )
    return response.json()["accessToken"]

def sync_schema_version(version: str, git_tag: str):
    """Sync a schema version to Port.io"""
    
    # Get schema metadata from crate
    entities = ["Motif", "Thread", "Bond"]  # Parse from crate
    components = ["QuantumState", "MotifContent", "BondContent"]
    
    # Get breaking changes from CHANGELOG
    breaking_changes = parse_changelog_breaking_changes(version)
    
    token = get_port_token()
    
    response = requests.post(
        "https://api.getport.io/v1/blueprints/schemaLibrary/entities",
        headers={"Authorization": f"Bearer {token}"},
        json={
            "identifier": f"familiar-schemas-{version}",
            "title": f"Familiar Schemas {version}",
            "properties": {
                "version": version,
                "gitTag": f"https://github.com/org/familiar-schemas/tree/{git_tag}",
                "publishedAt": datetime.utcnow().isoformat(),
                "breakingChanges": breaking_changes,
                "entities": entities,
                "components": components
            }
        }
    )
    
    return response.json()

# Sync all tags
tags = subprocess.check_output(["git", "tag", "-l", "v*"]).decode().split()
for tag in tags:
    sync_schema_version(tag.lstrip("v"), tag)
```

#### Auto-sync from CI/CD

```yaml
# .github/workflows/sync-to-port.yml
name: Sync to Port.io

on:
  push:
    tags:
      - 'v*'

jobs:
  sync:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Sync to Port.io
        uses: port-labs/port-github-action@v1
        with:
          clientId: ${{ secrets.PORT_CLIENT_ID }}
          clientSecret: ${{ secrets.PORT_CLIENT_SECRET }}
          operation: UPSERT
          identifier: familiar-schemas-${{ github.ref_name }}
          blueprint: schemaLibrary
          properties: |
            {
              "version": "${{ github.ref_name }}",
              "gitTag": "${{ github.server_url }}/${{ github.repository }}/tree/${{ github.ref_name }}",
              "publishedAt": "${{ github.event.head_commit.timestamp }}",
              "entities": ["Motif", "Thread", "Bond"],
              "components": ["QuantumState", "MotifContent"]
            }
```

---

### 3. Self-Service Actions (Scaffolding)

#### Action: Generate Service from Schema

```json
{
  "identifier": "generateService",
  "title": "Generate Service from Schema",
  "icon": "Microservice",
  "userInputs": {
    "properties": {
      "serviceName": {
        "title": "Service Name",
        "type": "string"
      },
      "schemaVersion": {
        "title": "Schema Version",
        "type": "string",
        "blueprint": "schemaLibrary",
        "format": "entity"
      },
      "entities": {
        "title": "Entities to Include",
        "type": "array",
        "items": {
          "type": "string",
          "blueprint": "schemaLibrary",
          "dataset": {
            "combinator": "and",
            "rules": [
              {
                "property": "$identifier",
                "operator": "=",
                "value": "{{.inputs.schemaVersion}}"
              }
            ]
          },
          "format": "entity-property",
          "entityPropertyPath": "entities"
        }
      },
      "templateType": {
        "title": "Template Type",
        "type": "string",
        "enum": ["microservice", "graphql-gateway"],
        "default": "microservice"
      }
    },
    "required": ["serviceName", "schemaVersion", "entities"]
  },
  "invocationMethod": {
    "type": "GITHUB",
    "org": "your-org",
    "repo": "familiar-infrastructure",
    "workflow": "generate-service.yml",
    "workflowInputs": {
      "service_name": "{{ .inputs.serviceName }}",
      "schema_version": "{{ .inputs.schemaVersion.identifier }}",
      "entities": "{{ .inputs.entities | join(\",\") }}",
      "template_type": "{{ .inputs.templateType }}"
    }
  }
}
```

#### GitHub Workflow (invoked by Port.io)

```yaml
# .github/workflows/generate-service.yml
name: Generate Service

on:
  workflow_dispatch:
    inputs:
      service_name:
        required: true
      schema_version:
        required: true
      entities:
        required: true
      template_type:
        required: true
      port_context:
        required: true

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - name: Generate Service
        run: |
          # Use Copier to generate service
          copier copy \
            templates/${{ inputs.template_type }} \
            ./${{ inputs.service_name }} \
            --data schema_version=${{ inputs.schema_version }} \
            --data entities=${{ inputs.entities }} \
            --data project_name=${{ inputs.service_name }}
      
      - name: Create Repository
        uses: actions/create-repository@v1
        with:
          name: ${{ inputs.service_name }}
          
      - name: Push Code
        run: |
          cd ${{ inputs.service_name }}
          git init
          git add .
          git commit -m "Generated from schema ${{ inputs.schema_version }}"
          git remote add origin https://github.com/your-org/${{ inputs.service_name }}
          git push -u origin main
      
      - name: Report to Port.io
        uses: port-labs/port-github-action@v1
        with:
          clientId: ${{ secrets.PORT_CLIENT_ID }}
          clientSecret: ${{ secrets.PORT_CLIENT_SECRET }}
          operation: UPSERT
          identifier: ${{ inputs.service_name }}
          blueprint: service
          properties: |
            {
              "language": "rust",
              "schemaVersion": "${{ inputs.schema_version }}",
              "entities": ${{ toJson(fromJson(inputs.entities)) }},
              "repository": "https://github.com/your-org/${{ inputs.service_name }}"
            }
          relations: |
            {
              "schemas": "${{ inputs.schema_version }}"
            }
```

---

### 4. Scorecards (Track Compliance)

#### Scorecard: Schema Version Compliance

```json
{
  "identifier": "schemaCompliance",
  "title": "Schema Version Compliance",
  "blueprint": "service",
  "rules": [
    {
      "identifier": "usingLatestSchema",
      "title": "Using Latest Schema Version",
      "level": "Gold",
      "query": {
        "combinator": "and",
        "rules": [
          {
            "property": "schemaUpToDate",
            "operator": "=",
            "value": true
          }
        ]
      }
    },
    {
      "identifier": "schemaWithin1Minor",
      "title": "Schema Within 1 Minor Version",
      "level": "Silver",
      "query": {
        "combinator": "and",
        "rules": [
          {
            "property": "$blueprint",
            "operator": "=",
            "value": "service"
          }
        ]
      }
    },
    {
      "identifier": "hasSchemaVersion",
      "title": "Has Schema Version Defined",
      "level": "Bronze",
      "query": {
        "combinator": "and",
        "rules": [
          {
            "property": "schemaVersion",
            "operator": "isNotEmpty"
          }
        ]
      }
    }
  ]
}
```

**Visual in Port.io:**
```
Service Scorecard
─────────────────────────────────────────────
motif-service                    🥇 Gold
  ✅ Using latest schema v0.1.0
  ✅ All tests passing
  ✅ Documentation up to date

thread-service                   🥇 Gold  
  ✅ Using latest schema v0.1.0
  ✅ All tests passing
  ✅ Documentation up to date

bond-service                     🥉 Bronze
  ⚠️  Using old schema v0.0.9
  ✅ Has schema version defined
  
  [Update Schema Version] button
─────────────────────────────────────────────
```

---

### 5. Dependency Visualization

Port.io automatically generates dependency graphs from relations:

```
Visual Graph in Port.io:

    ┌─────────────────────────┐
    │ familiar-schemas v0.1.0 │
    │ 🔵 Schema Library       │
    └────────┬────────────────┘
             │
    ┌────────┼────────┬────────┐
    │        │        │        │
    v        v        v        v
┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐
│motif│  │thread│ │bond │  │gql  │
│svc  │  │svc  │  │svc  │  │gate │
│✅   │  │✅   │  │⚠️   │  │✅   │
└─────┘  └─────┘  └─────┘  └─────┘
v0.1.0   v0.1.0   v0.0.9   v0.1.0
```

Click any node to:
- View properties
- See scorecard
- Trigger actions
- View related entities

---

### 6. Breaking Change Detection

#### Custom Action: Check Breaking Changes

```json
{
  "identifier": "checkBreakingChanges",
  "title": "Check Breaking Changes",
  "userInputs": {
    "properties": {
      "fromVersion": {
        "title": "Current Version",
        "type": "string",
        "blueprint": "schemaLibrary",
        "format": "entity"
      },
      "toVersion": {
        "title": "Target Version",
        "type": "string",
        "blueprint": "schemaLibrary",
        "format": "entity"
      }
    }
  },
  "invocationMethod": {
    "type": "WEBHOOK",
    "url": "https://your-api.com/check-breaking-changes",
    "method": "POST",
    "body": {
      "from": "{{ .inputs.fromVersion.identifier }}",
      "to": "{{ .inputs.toVersion.identifier }}"
    }
  }
}
```

#### Webhook Handler

```rust
// breaking-change-api/src/main.rs
use axum::{Json, extract::Json as ExtractJson};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CheckRequest {
    from: String,
    to: String,
}

#[derive(Serialize)]
struct BreakingChange {
    entity: String,
    field: String,
    change_type: String,
    severity: String,
}

async fn check_breaking_changes(
    ExtractJson(req): ExtractJson<CheckRequest>
) -> Json<Vec<BreakingChange>> {
    // Load both schema versions
    let from_schemas = load_schemas(&req.from);
    let to_schemas = load_schemas(&req.to);
    
    // Compare
    let changes = compare_schemas(&from_schemas, &to_schemas);
    
    // Return breaking changes
    Json(changes.into_iter()
        .filter(|c| c.severity == "breaking")
        .collect())
}
```

**Result in Port.io:**
```
Breaking Changes: v0.0.9 → v0.1.0
─────────────────────────────────────
❌ Motif.pattern removed
   Impact: High
   Affected: 2 services
   
⚠️  Thread.status enum changed
   Impact: Medium
   Affected: 1 service

[Generate Migration PRs] [Notify Teams]
─────────────────────────────────────
```

---

## Port.io UI Examples

### 1. Software Catalog View

```
┌─────────────────────────────────────────────────┐
│ Software Catalog                      [+ New]   │
├─────────────────────────────────────────────────┤
│                                                 │
│ 📦 Schema Libraries (3)                        │
│   ├─ familiar-schemas v0.1.0  🟢 Latest       │
│   ├─ familiar-schemas v0.0.9  🟡 Old          │
│   └─ familiar-schemas v0.0.8  🔴 Deprecated   │
│                                                 │
│ 🔧 Services (4)                                │
│   ├─ motif-service        [v0.1.0] 🥇 Gold    │
│   ├─ thread-service       [v0.1.0] 🥇 Gold    │
│   ├─ bond-service         [v0.0.9] 🥉 Bronze  │
│   └─ graphql-gateway      [v0.1.0] 🥇 Gold    │
│                                                 │
│ 📝 Templates (3)                               │
│   ├─ Microservice Template                    │
│   ├─ GraphQL Gateway Template                 │
│   └─ Event Processor Template                 │
└─────────────────────────────────────────────────┘
```

### 2. Entity Detail View

```
┌─────────────────────────────────────────────────┐
│ familiar-schemas v0.1.0                         │
├─────────────────────────────────────────────────┤
│                                                 │
│ Properties:                                     │
│   Version: v0.1.0                              │
│   Published: 2025-01-06                        │
│   Git Tag: github.com/org/familiar-schemas/... │
│                                                 │
│ Entities (3):                                   │
│   • Motif                                      │
│   • Thread                                     │
│   • Bond                                       │
│                                                 │
│ Components (5):                                 │
│   • QuantumState                               │
│   • MotifContent                               │
│   • BondContent                                │
│   • ConsolidationState                         │
│   • ThreadContent                              │
│                                                 │
│ Breaking Changes:                               │
│   ❌ Motif.pattern field removed               │
│   ⚠️  Thread.status enum changed               │
│                                                 │
│ Consumed By (3 services):                      │
│   → motif-service                              │
│   → thread-service                             │
│   → graphql-gateway                            │
│                                                 │
│ [Generate Service] [View Docs] [Compare]      │
└─────────────────────────────────────────────────┘
```

### 3. Self-Service Action UI

```
┌─────────────────────────────────────────────────┐
│ Generate Service from Schema                    │
├─────────────────────────────────────────────────┤
│                                                 │
│ Service Name *                                  │
│ [my-new-service_________________]              │
│                                                 │
│ Schema Version *                                │
│ ○ v0.1.0 (latest) ✅                           │
│ ○ v0.0.9                                       │
│                                                 │
│ Entities to Include *                           │
│ ☑ Motif                                        │
│ ☑ Thread                                       │
│ ☐ Bond                                         │
│                                                 │
│ Template Type *                                 │
│ ● Microservice                                 │
│ ○ GraphQL Gateway                              │
│ ○ Event Processor                              │
│                                                 │
│              [Cancel]  [Generate] ──→          │
└─────────────────────────────────────────────────┘
```

---

## Integration with Existing Tools

### Port.io + GitHub Actions

```yaml
# Service repo: .github/workflows/update-port.yml
name: Update Port.io

on:
  push:
    branches: [main]

jobs:
  update:
    runs-on: ubuntu-latest
    steps:
      - name: Extract schema version
        id: schema
        run: |
          VERSION=$(cargo metadata --format-version=1 | \
            jq -r '.packages[] | select(.name=="familiar-schemas") | .version')
          echo "version=$VERSION" >> $GITHUB_OUTPUT
      
      - name: Update Port.io
        uses: port-labs/port-github-action@v1
        with:
          clientId: ${{ secrets.PORT_CLIENT_ID }}
          clientSecret: ${{ secrets.PORT_CLIENT_SECRET }}
          operation: PATCH
          identifier: ${{ github.event.repository.name }}
          blueprint: service
          properties: |
            {
              "schemaVersion": "${{ steps.schema.outputs.version }}",
              "lastDeployed": "${{ github.event.head_commit.timestamp }}"
            }
```

### Port.io + Cargo

```toml
# In service Cargo.toml, add metadata
[package.metadata.port]
blueprint = "service"
schema-version = "0.1.0"
entities = ["Motif", "Thread"]
```

---

## Advanced: Custom Widgets

Port.io allows custom React widgets:

```tsx
// widgets/SchemaVersionMatrix.tsx
export const SchemaVersionMatrix = () => {
  const { entities } = usePort();
  
  const services = entities.filter(e => e.blueprint === 'service');
  const schemas = entities.filter(e => e.blueprint === 'schemaLibrary');
  
  return (
    <Table>
      <TableHead>
        <TableRow>
          <TableCell>Service</TableCell>
          {schemas.map(schema => (
            <TableCell key={schema.id}>{schema.properties.version}</TableCell>
          ))}
        </TableRow>
      </TableHead>
      <TableBody>
        {services.map(service => (
          <TableRow key={service.id}>
            <TableCell>{service.title}</TableCell>
            {schemas.map(schema => (
              <TableCell>
                {service.properties.schemaVersion === schema.properties.version 
                  ? '✅' 
                  : ''}
              </TableCell>
            ))}
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
};
```

---

## Quick Start Checklist

- [ ] Sign up for Port.io (https://app.getport.io)
- [ ] Create blueprints (Schema Library, Service, Template)
- [ ] Set up GitHub integration
- [ ] Add familiar-schemas to catalog
- [ ] Create self-service action for service generation
- [ ] Set up scorecard for schema compliance
- [ ] Configure webhook for breaking change detection
- [ ] Test: Generate a service through Port.io UI

**Time to working system:** 4-8 hours

---

## Port.io vs Backstage

| Feature | Port.io | Backstage |
|---------|---------|-----------|
| **Setup Time** | 4-8 hours | 2-3 days |
| **Hosting** | SaaS (or self-hosted) | Self-hosted only |
| **Dependency Graph** | ✅ Native, beautiful | ⚠️ Plugin required |
| **Self-Service Actions** | ✅ Built-in | ⚠️ Custom code |
| **Scorecards** | ✅ Native | ❌ Custom plugin |
| **Breaking Changes** | ✅ Via webhooks | ⚠️ Custom plugin |
| **Customization** | ⚠️ Limited | ✅ Full (React) |
| **Cost** | $$$ (SaaS) | $ (hosting only) |

**For your use case:** Port.io wins on ease of use and out-of-the-box features.

---

## Resources

- **Port.io:** https://www.getport.io
- **Docs:** https://docs.getport.io
- **GitHub Action:** https://github.com/port-labs/port-github-action
- **Examples:** https://github.com/port-labs/port-examples
- **Community:** https://github.com/orgs/port-labs/discussions

---

## Next Steps

1. **Sign up** for Port.io
2. **Review** `PORT_IO_INTEGRATION.md` (this file)
3. **Create** blueprints from examples above
4. **Sync** familiar-schemas to catalog
5. **Test** self-service action
6. **Deploy** to team

**Result:** Click-to-build system with visual dependency tracking in 4-8 hours!


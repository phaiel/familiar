# Quick Start: Port.io Integration

**Goal:** Get visual dependency tracking and click-to-build working in 4 hours

---

## Prerequisites

- [ ] Port.io account (free tier: https://app.getport.io/signup)
- [ ] GitHub repository with familiar-schemas
- [ ] GitHub Personal Access Token

---

## Step 1: Create Blueprints (10 mins)

### 1.1 Schema Library Blueprint

In Port.io UI → Builder → Blueprints → Create New:

```json
{
  "identifier": "schemaLibrary",
  "title": "Schema Library",
  "icon": "Blueprint",
  "schema": {
    "properties": {
      "version": {
        "type": "string",
        "title": "Version"
      },
      "entities": {
        "type": "array",
        "title": "Entities"
      }
    }
  },
  "relations": {
    "consumedBy": {
      "target": "service",
      "many": true
    }
  }
}
```

### 1.2 Service Blueprint

```json
{
  "identifier": "service",
  "title": "Service",
  "icon": "Microservice",
  "schema": {
    "properties": {
      "schemaVersion": {
        "type": "string",
        "title": "Schema Version"
      }
    }
  },
  "relations": {
    "schemas": {
      "target": "schemaLibrary",
      "required": true
    }
  }
}
```

---

## Step 2: Add Schema to Catalog (5 mins)

Port.io UI → Catalog → schemaLibrary → Add Entity:

```json
{
  "identifier": "familiar-schemas-v0.1.0",
  "title": "Familiar Schemas v0.1.0",
  "properties": {
    "version": "v0.1.0",
    "entities": ["Motif", "Thread", "Bond"]
  }
}
```

---

## Step 3: Create Self-Service Action (30 mins)

Port.io UI → Self-Service → Create Action:

**Name:** Generate Service from Schema

**Blueprint:** service

**User Inputs:**
```json
{
  "serviceName": {
    "type": "string",
    "title": "Service Name"
  },
  "schemaVersion": {
    "type": "string",
    "title": "Schema Version",
    "blueprint": "schemaLibrary",
    "format": "entity"
  }
}
```

**Invocation Method:** GitHub Workflow

**Workflow:** `generate-service.yml`

---

## Step 4: Create GitHub Workflow (30 mins)

In your infrastructure repo:

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
      port_context:
        required: true

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Generate with Copier
        run: |
          pip install copier
          copier copy templates/microservice ./${{ inputs.service_name }} \
            --data schema_version=${{ inputs.schema_version }} \
            --data project_name=${{ inputs.service_name }} \
            --force
      
      - name: Report to Port
        uses: port-labs/port-github-action@v1
        with:
          clientId: ${{ secrets.PORT_CLIENT_ID }}
          clientSecret: ${{ secrets.PORT_CLIENT_SECRET }}
          operation: UPSERT
          identifier: ${{ inputs.service_name }}
          blueprint: service
          properties: |
            {
              "schemaVersion": "${{ inputs.schema_version }}"
            }
          relations: |
            {
              "schemas": "${{ inputs.schema_version }}"
            }
```

---

## Step 5: Test It! (5 mins)

1. Go to Port.io → Self-Service
2. Click "Generate Service from Schema"
3. Fill in:
   - Service Name: `test-service`
   - Schema Version: `familiar-schemas-v0.1.0`
4. Click "Execute"
5. Watch the magic! ✨

---

## Step 6: View Dependencies (2 mins)

Port.io → Catalog → familiar-schemas-v0.1.0 → Dependencies tab

You should see:
```
familiar-schemas-v0.1.0
  └─→ test-service
```

---

## Step 7: Add Scorecard (20 mins)

Port.io → Scorecards → Create Scorecard:

**Name:** Schema Compliance
**Blueprint:** service

**Rules:**
- ✅ Gold: Using latest schema
- ⚠️ Silver: Within 1 minor version
- 🥉 Bronze: Has schema defined

```json
{
  "rules": [
    {
      "identifier": "latest",
      "title": "Using Latest Schema",
      "level": "Gold",
      "query": {
        "property": "schemaVersion",
        "operator": "=",
        "value": "v0.1.0"
      }
    }
  ]
}
```

---

## Step 8: Auto-Sync from CI/CD (30 mins)

Add to familiar-schemas repo:

```yaml
# .github/workflows/sync-to-port.yml
name: Sync to Port

on:
  push:
    tags: ['v*']

jobs:
  sync:
    runs-on: ubuntu-latest
    steps:
      - uses: port-labs/port-github-action@v1
        with:
          clientId: ${{ secrets.PORT_CLIENT_ID }}
          clientSecret: ${{ secrets.PORT_CLIENT_SECRET }}
          operation: UPSERT
          identifier: familiar-schemas-${{ github.ref_name }}
          blueprint: schemaLibrary
          properties: |
            {
              "version": "${{ github.ref_name }}",
              "entities": ["Motif", "Thread", "Bond"]
            }
```

---

## Done! 🎉

You now have:
- ✅ Visual dependency graph
- ✅ Click-to-build services
- ✅ Schema version tracking
- ✅ Compliance scorecards
- ✅ Auto-sync from CI/CD

**Total time:** ~2-4 hours

---

## Next Steps

1. Add more services to catalog
2. Create more self-service actions
3. Build breaking change detector
4. Add custom widgets
5. Train team on Port.io

---

## Troubleshooting

**Q: GitHub workflow not triggering?**
A: Check Port.io → Settings → GitHub integration is connected

**Q: Can't see dependencies?**
A: Make sure relations are set in both blueprints

**Q: Scorecard not working?**
A: Verify the property paths match your blueprint schema

---

## Resources

- **Port.io Docs:** https://docs.getport.io
- **Full Integration Guide:** `PORT_IO_INTEGRATION.md`
- **GitHub Action:** https://github.com/port-labs/port-github-action


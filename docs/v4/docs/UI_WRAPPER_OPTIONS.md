# UI Wrapper Options for Schema-First Architecture

**Question:** Is there a UI that can visualize schema dependencies and manage conflicts between generated code and schemas?

**Answer:** Yes! Several options exist, with **Backstage** being the ideal fit for this architecture.

---

## 🎯 Recommended: Port.io

**Perfect fit for your use case!** (Better than Backstage for faster setup)

### What Is Port.io?

Port.io is a modern developer portal (SaaS or self-hosted) that:
- ✅ Visualizes service dependencies
- ✅ Manages software templates (like Copier)
- ✅ Tracks schema versions across services
- ✅ Detects breaking changes
- ✅ Provides "click-to-scaffold" UI
- ✅ Integrates with CI/CD

**Website:** https://backstage.io

### Why It Fits Perfectly

Your architecture:
```
familiar-schemas (versioned) 
  → Templates (Copier)
  → Generated Services
```

Backstage provides:
```
Software Catalog
  ├─ Schema Library Component (tracks versions)
  ├─ Template Catalog (your Copier templates)
  ├─ Generated Services (tracks which schema version)
  └─ Dependency Graph Visualization
```

### Key Features for Your Use Case

#### 1. Software Catalog

Track schema versions and consumers:

```yaml
# catalog-info.yaml for familiar-schemas
apiVersion: backstage.io/v1alpha1
kind: Component
metadata:
  name: familiar-schemas
  title: Familiar Schema Library
  description: Canonical schema definitions
  tags:
    - schemas
    - rust
spec:
  type: library
  lifecycle: production
  owner: platform-team
  providesApis:
    - familiar-schemas-api
  system: familiar
```

Track services using schemas:

```yaml
# catalog-info.yaml for motif-service
apiVersion: backstage.io/v1alpha1
kind: Component
metadata:
  name: motif-service
spec:
  type: service
  dependsOn:
    - component:familiar-schemas@v0.1.0  # Tracks schema version!
  consumesApis:
    - familiar-schemas-api
```

#### 2. Software Templates (Replaces/Wraps Copier)

```yaml
# template.yaml
apiVersion: scaffolder.backstage.io/v1beta3
kind: Template
metadata:
  name: familiar-microservice
  title: Familiar Microservice
  description: Generate a microservice from Familiar schemas
spec:
  owner: platform-team
  type: service
  
  parameters:
    - title: Service Configuration
      required:
        - name
        - schema_version
      properties:
        name:
          title: Service Name
          type: string
        schema_version:
          title: Schema Version
          type: string
          enum:
            - v0.1.0
            - v0.2.0
          default: v0.1.0
        entities:
          title: Which entities to include?
          type: array
          items:
            type: string
            enum:
              - Motif
              - Thread
              - Bond
  
  steps:
    - id: fetch-schemas
      name: Fetch Schema Library
      action: fetch:template
      input:
        url: https://github.com/org/familiar-schemas
        targetPath: ./schemas
        values:
          version: ${{ parameters.schema_version }}
    
    - id: copier
      name: Apply Copier Template
      action: copier:run
      input:
        template: ./templates/microservice
        data:
          project_name: ${{ parameters.name }}
          schema_version: ${{ parameters.schema_version }}
          entities: ${{ parameters.entities }}
    
    - id: register
      name: Register in Catalog
      action: catalog:register
      input:
        repoContentsUrl: ${{ steps.publish.output.repoContentsUrl }}
        catalogInfoPath: '/catalog-info.yaml'
```

**UI for this:**

![Backstage Template UI](https://backstage.io/img/assets/software-templates/software-template-form.png)

#### 3. Dependency Graph Visualization

Backstage shows:
```
familiar-schemas v0.1.0
  ├── motif-service (uses v0.1.0) ✅
  ├── thread-service (uses v0.1.0) ✅
  ├── bond-service (uses v0.0.9) ⚠️  OUTDATED
  └── graphql-gateway (uses v0.1.0) ✅
```

Visual dependency graph:
- See which services use which schema version
- Identify services that need updating
- Track breaking change impact

#### 4. TechDocs (Documentation)

Generates docs from your schema crate:

```yaml
# mkdocs.yml in familiar-schemas repo
site_name: Familiar Schemas
docs_dir: docs
plugins:
  - techdocs-core
```

Backstage automatically:
- Builds documentation
- Displays it in the portal
- Updates on schema changes

#### 5. Schema Diff Plugin (Custom)

You can build a custom Backstage plugin to:

```typescript
// SchemaVersionDiffPlugin.tsx
export const SchemaVersionDiffPlugin = () => {
  const compareVersions = async (v1: string, v2: string) => {
    // Compare JSON Schemas
    const diff = await api.compareSchemas(v1, v2);
    
    return (
      <SchemaChanges 
        breaking={diff.breaking}
        additions={diff.additions}
        removals={diff.removals}
      />
    );
  };
};
```

Visual diff showing:
- ❌ Breaking changes (field removals, type changes)
- ⚠️  Deprecations
- ✅ Additions (new optional fields)

#### 6. API Browser

Browse schemas interactively:

```yaml
# familiar-schemas-api.yaml
apiVersion: backstage.io/v1alpha1
kind: API
metadata:
  name: familiar-schemas-api
  description: Familiar canonical schemas
spec:
  type: library
  lifecycle: production
  owner: platform-team
  definition:
    $text: https://github.com/org/familiar-schemas/generated/openapi.yaml
```

Backstage displays:
- All entity types
- Field definitions
- Validation rules
- Examples

---

## 🎨 Visual Example: Backstage UI for Your System

### 1. Software Catalog View

```
╔══════════════════════════════════════════════════════════════╗
║ Familiar Schema Library                                      ║
║ Version: v0.1.0                                             ║
║ Owner: Platform Team                                        ║
║                                                             ║
║ Dependencies:                                               ║
║   • serde 1.0                                              ║
║   • schemars 0.8                                           ║
║                                                             ║
║ Used By: (5 services)                                      ║
║   • motif-service (v0.1.0) ✅                              ║
║   • thread-service (v0.1.0) ✅                             ║
║   • bond-service (v0.0.9) ⚠️  OUTDATED                     ║
║   • graphql-gateway (v0.1.0) ✅                            ║
║   • client-sdk (v0.1.0) ✅                                 ║
║                                                             ║
║ [View Docs] [View API] [Create Service]                   ║
╚══════════════════════════════════════════════════════════════╝
```

### 2. Template Scaffolder

```
╔══════════════════════════════════════════════════════════════╗
║ Create New Familiar Microservice                            ║
╠══════════════════════════════════════════════════════════════╣
║                                                             ║
║ Service Name: [my-new-service_____________]                ║
║                                                             ║
║ Schema Version: [v0.1.0 ▼]                                 ║
║   • v0.1.0 (latest) ✅                                      ║
║   • v0.0.9                                                  ║
║                                                             ║
║ Entities to Include: ☑ Motif  ☑ Thread  ☐ Bond            ║
║                                                             ║
║ Template: [Microservice ▼]                                 ║
║   • Microservice (REST API)                                ║
║   • GraphQL Service                                        ║
║   • Event Processor                                        ║
║                                                             ║
║ [Preview] [Generate Service]                               ║
╚══════════════════════════════════════════════════════════════╝
```

### 3. Dependency Graph

```
        ┌─────────────────────┐
        │ familiar-schemas    │
        │ v0.1.0              │
        └──────────┬──────────┘
                   │
        ┌──────────┼──────────┬──────────┐
        │          │          │          │
        v          v          v          v
  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
  │ motif   │ │ thread  │ │ bond    │ │ graphql │
  │ service │ │ service │ │ service │ │ gateway │
  │ v0.1.0✅│ │ v0.1.0✅│ │ v0.0.9⚠️ │ │ v0.1.0✅│
  └─────────┘ └─────────┘ └─────────┘ └─────────┘
```

Click any node to:
- See schema version compatibility
- View breaking changes
- Generate update PR

### 4. Schema Version Comparison

```
╔══════════════════════════════════════════════════════════════╗
║ Compare Schema Versions                                      ║
║ v0.0.9  →  v0.1.0                                           ║
╠══════════════════════════════════════════════════════════════╣
║                                                             ║
║ ❌ Breaking Changes (2)                                      ║
║   • Motif.pattern: field removed                           ║
║   • Thread.status: enum value changed                      ║
║                                                             ║
║ ⚠️  Deprecations (1)                                        ║
║   • Bond.legacy_id: marked deprecated                      ║
║                                                             ║
║ ✅ Additions (3)                                            ║
║   • Motif.quantum_state: new optional field                ║
║   • Thread.metadata: new optional field                    ║
║   • New entity: Shuttle                                    ║
║                                                             ║
║ Affected Services: (1)                                     ║
║   • bond-service (needs update)                           ║
║                                                             ║
║ [Generate Migration Guide] [Create Update PRs]            ║
╚══════════════════════════════════════════════════════════════╝
```

---

## 🔧 Alternative Options

### 2. Nx Console (for Monorepos)

**Good if:** You keep all services in one monorepo

**Features:**
- Visual dependency graph
- Build orchestration
- Affected service detection

**Not ideal for:**
- Cross-repo dependencies
- Schema versioning tracking

### 3. Schema Registry UI (Confluent, Apicurio)

**Good if:** You want pure schema management

**Features:**
- Schema versioning
- Compatibility checking
- Breaking change detection

**Not ideal for:**
- Template scaffolding
- Service tracking

### 4. Custom Web UI

Build your own using:
- **Frontend:** React + D3.js (for graphs)
- **Backend:** Rust + Axum
- **Features:**
  - Parse `familiar-schemas` crate
  - Visualize dependencies
  - Show version compatibility
  - Detect conflicts

**Example structure:**

```
schema-portal/
├── frontend/
│   ├── src/
│   │   ├── components/
│   │   │   ├── DependencyGraph.tsx
│   │   │   ├── SchemaViewer.tsx
│   │   │   └── VersionCompare.tsx
│   │   └── App.tsx
│   └── package.json
└── backend/
    ├── src/
    │   ├── main.rs
    │   ├── schema_parser.rs
    │   └── dependency_analyzer.rs
    └── Cargo.toml
```

---

## 📊 Feature Comparison

| Feature | Backstage | Nx Console | Schema Registry | Custom |
|---------|-----------|------------|-----------------|--------|
| **Dependency Graph** | ✅ Visual | ✅ Visual | ❌ No | ✅ Custom |
| **Template Scaffolding** | ✅ Built-in | ⚠️ Limited | ❌ No | ✅ Custom |
| **Schema Versioning** | ✅ Via catalog | ❌ No | ✅ Native | ✅ Custom |
| **Breaking Change Detection** | ✅ Via plugin | ❌ No | ✅ Native | ✅ Custom |
| **Service Tracking** | ✅ Software Catalog | ✅ Monorepo | ❌ No | ✅ Custom |
| **CI/CD Integration** | ✅ Native | ✅ Native | ⚠️ Limited | ✅ Custom |
| **Setup Complexity** | Medium | Low | Low | High |
| **Customization** | ✅ Plugins | ⚠️ Limited | ⚠️ Limited | ✅✅ Full |

---

## 🎯 Recommendation: Backstage + Custom Plugins

### Phase 1: Basic Backstage Setup

```bash
# Install Backstage
npx @backstage/create-app

# Add familiar-schemas to catalog
# Add templates
# Configure CI/CD
```

**Time:** 1-2 days  
**Benefit:** Template scaffolding, basic dependency tracking

### Phase 2: Schema Registry Plugin

Create custom plugin:

```typescript
// plugins/familiar-schemas/src/plugin.ts
export const familiarSchemasPlugin = createPlugin({
  id: 'familiar-schemas',
  routes: {
    root: rootRouteRef,
  },
});

export const FamiliarSchemasPage = familiarSchemasPlugin.provide(
  createRoutableExtension({
    name: 'FamiliarSchemasPage',
    component: () =>
      import('./components/SchemaExplorer').then(m => m.SchemaExplorer),
    mountPoint: rootRouteRef,
  }),
);
```

Features:
- Browse all schemas
- View versions
- See consumers
- Detect breaking changes

**Time:** 1 week  
**Benefit:** Full schema visibility

### Phase 3: Conflict Detection

```typescript
// ConflictDetector.tsx
export const ConflictDetector = () => {
  const conflicts = useConflicts();
  
  return (
    <List>
      {conflicts.map(conflict => (
        <ConflictItem
          service={conflict.service}
          schemaVersion={conflict.schemaVersion}
          latestVersion={conflict.latestVersion}
          breakingChanges={conflict.breakingChanges}
          onResolve={() => generateUpdatePR(conflict)}
        />
      ))}
    </List>
  );
};
```

**Time:** 1 week  
**Benefit:** Automated conflict detection and resolution

---

## 🚀 Quick Start: Backstage for Familiar

### 1. Install Backstage

```bash
npx @backstage/create-app
cd my-backstage-app
yarn dev
```

### 2. Add Familiar Schemas to Catalog

```yaml
# catalog-entities/familiar-schemas.yaml
apiVersion: backstage.io/v1alpha1
kind: Component
metadata:
  name: familiar-schemas
  title: Familiar Schema Library
  description: Canonical schema definitions
  annotations:
    github.com/project-slug: your-org/familiar-schemas
    backstage.io/techdocs-ref: dir:.
spec:
  type: library
  lifecycle: production
  owner: platform-team
  system: familiar
```

### 3. Create Template

```yaml
# templates/familiar-microservice/template.yaml
apiVersion: scaffolder.backstage.io/v1beta3
kind: Template
metadata:
  name: familiar-microservice
  title: Familiar Microservice
spec:
  parameters:
    - title: Configuration
      properties:
        name:
          title: Service Name
          type: string
        schema_version:
          title: Schema Version
          type: string
          enum: [v0.1.0, v0.2.0]
  
  steps:
    - id: copier
      name: Generate Service
      action: run:command
      input:
        command: |
          copier copy templates/microservice ${{ parameters.name }} \
            --data schema_version=${{ parameters.schema_version }}
```

### 4. View in Backstage

Navigate to:
- `http://localhost:3000/catalog` - See familiar-schemas component
- `http://localhost:3000/create` - Use template to create service
- `http://localhost:3000/docs` - Read generated docs

---

## 💡 Key Insights

### Why Backstage Wins

1. **Built for This:** Designed for template-driven development
2. **Schema Aware:** Software catalog tracks versions
3. **Extensible:** Plugins for custom schema visualization
4. **Industry Standard:** Used by Spotify, Netflix, many others
5. **Active Community:** Lots of plugins and support

### What You Get

```
Without Backstage:
  • Manual schema version tracking
  • Terminal-based template execution
  • No visual dependency graph
  • Manual conflict detection

With Backstage:
  • ✅ Visual schema catalog
  • ✅ Click-to-scaffold services
  • ✅ Interactive dependency graph
  • ✅ Automated conflict detection
  • ✅ Breaking change notifications
  • ✅ Version compatibility matrix
```

---

## 📚 Resources

### Backstage
- **Website:** https://backstage.io
- **Docs:** https://backstage.io/docs
- **Plugins:** https://backstage.io/plugins
- **GitHub:** https://github.com/backstage/backstage

### Schema Management Plugins
- **OpenAPI Plugin:** https://github.com/backstage/backstage/tree/master/plugins/api-docs
- **TechDocs:** https://backstage.io/docs/features/techdocs

### Custom Plugin Development
- **Plugin Guide:** https://backstage.io/docs/plugins/create-a-plugin
- **Template Actions:** https://backstage.io/docs/features/software-templates/writing-custom-actions

---

## 🎬 Next Steps

1. **Try Backstage** (1 day)
   ```bash
   npx @backstage/create-app
   ```

2. **Add Familiar Schemas** (2 hours)
   - Create catalog-info.yaml
   - Register in Backstage

3. **Create First Template** (4 hours)
   - Wrap your Copier template
   - Test scaffolding

4. **Build Dependency Plugin** (1 week)
   - Visualize schema → service relationships
   - Track versions

5. **Add Conflict Detection** (1 week)
   - Compare schema versions
   - Detect breaking changes
   - Generate migration PRs

**Total time to full system:** ~2-3 weeks

---

## Bottom Line

**Yes, there's a perfect UI wrapper: Backstage**

It provides:
- ✅ Visual dependency graphs
- ✅ Template-driven scaffolding
- ✅ Schema version tracking
- ✅ Conflict detection (via plugins)
- ✅ Breaking change visualization
- ✅ Click-to-build functionality

**Recommendation:** Start with Backstage, add custom plugins for schema-specific features.

**Alternative:** Build custom UI if you need very specific workflow, but Backstage covers 80% of needs out of the box.


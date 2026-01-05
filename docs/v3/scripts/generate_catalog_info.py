#!/usr/bin/env python3
"""
Generate Backstage catalog-info.yaml from JSON schemas.

This script analyzes all JSON schema files in the schemas directory and generates
individual Backstage catalog components with proper dependency relationships
based on $ref analysis.
"""

import json
import os
import re
import yaml
from pathlib import Path
from typing import Dict, List, Set, Optional, Any
from urllib.parse import urlparse
import argparse


class SchemaCatalogGenerator:
    def __init__(self, schemas_dir: str = "schemas"):
        # Resolve schemas_dir relative to this script's location
        script_dir = Path(__file__).parent
        self.schemas_dir = (script_dir / schemas_dir).resolve()
        
        self.schemas: Dict[str, Dict] = {}
        self.schema_files: Dict[str, str] = {}  # component_name -> file_path
        self.dependencies: Dict[str, Set[str]] = {}  # component_name -> set of dependencies
        
        # The 23 production-ready schemas that generate Rust code
        self.production_schemas = {
            'Bond.schema.json', 'BondStateChangeRequestedEvent.schema.json', 'BondStateLogTable.schema.json',
            'CoursePayload.schema.json', 'DraftPayload.schema.json', 'Filament.schema.json',
            'Focus.schema.json', 'IngestionWorkflow.schema.json', 'Intent.schema.json',
            'Moment.schema.json', 'MomentAccessedEvent.schema.json', 'Motif.schema.json',
            'ReconciliationResultPayload.schema.json', 'ShuttlePayload.schema.json', 'Stitch.schema.json',
            'StitchInteractionRequest.schema.json', 'StitchInteractionResponse.schema.json',
            'Tenant.schema.json', 'Thread.schema.json', 'ThreadStateChangeRequestedEvent.schema.json',
            'ThreadStateLogTable.schema.json', 'WeavePayload.schema.json', 'WeaveUnitPayload.schema.json'
        }
        self.system_mapping = {
            'api': {
                'system': 'familiar-physics-engine',
                'owner': 'team-platform-infrastructure',
                'dependsOn': ['component:default/physics-api-gateway']
            },
            'event': {
                'system': 'familiar-physics-engine',
                'owner': 'team-platform-infrastructure',
                'dependsOn': ['resource:default/redpanda-cluster']
            },
            'payload': {
                'system': 'familiar-physics-engine',
                'owner': 'team-cognitive-modeling',
                'dependsOn': []
            },
            'entity': {
                'system': 'familiar-physics-engine',
                'owner': 'team-cognitive-modeling',
                'dependsOn': ['resource:default/timescaledb-physics-db']
            },
            'component': {
                'system': 'familiar-physics-engine',
                'owner': 'team-physics-core',
                'dependsOn': ['component:default/physics-engine-core']
            },
            'law': {
                'system': 'familiar-physics-engine',
                'owner': 'team-physics-core',
                'dependsOn': ['component:default/classical-physics-service']
            },
            'table': {
                'system': 'familiar-schema-system',
                'owner': 'team-platform-infrastructure',
                'dependsOn': ['resource:default/timescaledb-physics-db']
            },
            'foundation': {
                'system': 'familiar-schema-system',
                'owner': 'team-system-architecture',
                'dependsOn': []
            },
            'snippet': {
                'system': 'familiar-schema-system',
                'owner': 'team-system-architecture',
                'dependsOn': []
            },
            'default': {
                'system': 'familiar-schema-system',
                'owner': 'team-system-architecture',
                'dependsOn': []
            }
        }
        
    def load_all_schemas(self):
        """Load all JSON schema files from the schemas directory."""
        print(f"Loading schemas from {self.schemas_dir}")
        
        for schema_file in self.schemas_dir.rglob("*.json"):
            if schema_file.name.startswith('.'):
                continue
            
            # Skip assembled schemas if we have the source version
            # (prefer source schemas for documentation purposes)
            relative_path = schema_file.relative_to(self.schemas_dir)
            if 'assembled/' in str(relative_path):
                source_path = self.schemas_dir / str(relative_path).replace('assembled/', '')
                if source_path.exists():
                    continue  # Skip assembled version, use source
                
            try:
                with open(schema_file, 'r') as f:
                    schema_data = json.load(f)
                
                component_name = self.file_path_to_component_name(schema_file)
                self.schemas[component_name] = schema_data
                # Use a path relative to the project root for consistency
                self.schema_files[component_name] = str(schema_file.relative_to(self.schemas_dir.parent.parent))
                
                print(f"  Loaded: {component_name} <- {schema_file.relative_to(self.schemas_dir)}")
                
            except Exception as e:
                print(f"  Error loading {schema_file}: {e}")
    
    def file_path_to_component_name(self, file_path: Path) -> str:
        """Convert a file path to a Backstage component name."""
        # Remove .schema.json or other extensions and convert to kebab-case
        name = file_path.name.split('.')[0]
        
        # Replace underscores with hyphens for consistency
        name = name.replace('_', '-')
        
        # Convert CamelCase to kebab-case
        name = re.sub(r'([a-z0-9])([A-Z])', r'\1-\2', name).lower()
        
        # Add directory context for disambiguation
        relative_path = file_path.relative_to(self.schemas_dir)
        # Find the top-level directory inside schemas/
        if len(relative_path.parts) > 1:
            directory = relative_path.parts[0]
            if directory not in ["_base", "assembled"]:  # _base is foundational, assembled is a dist dir
                # Prepend directory to avoid name clashes, e.g., entities-bond vs components-bond
                name = f"{directory}-{name}"
        
        # Ensure 'base' prefix for schemas in the _base directory for clarity
        if '_base/' in str(relative_path) and not name.startswith('base-'):
            name = f'base-{name}'

        return name
    
    def extract_ref_dependencies(self, schema: Dict, component_name: str) -> Set[str]:
        """Extract $ref dependencies from a schema and convert to component names."""
        refs = set()
        
        def find_refs(obj):
            if isinstance(obj, dict):
                for key, value in obj.items():
                    if key == "$ref" and isinstance(value, str):
                        refs.add(value)
                    else:
                        find_refs(value)
            elif isinstance(obj, list):
                for item in obj:
                    find_refs(item)
        
        find_refs(schema)
        
        # Convert $ref paths to component names
        dependencies = set()
        for ref in refs:
            dep_component = self.ref_to_component_name(ref, component_name)
            if dep_component and dep_component != component_name:
                dependencies.add(dep_component)
        
        return dependencies
    
    def ref_to_component_name(self, ref: str, current_component: str) -> Optional[str]:
        """Convert a $ref path to a component name."""
        # Strip URL fragment if present
        if '#/' in ref:
            ref = ref.split('#/')[0]
        
        # Handle relative refs like "./BaseMetadata.schema.json" or "../_base/BaseComponent.schema.json"
        if ref.startswith('./') or ref.startswith('../'):
            # Normalize the path and extract filename
            ref_path = Path(ref)
            filename = ref_path.name
            
            # Remove .schema.json extension
            if filename.endswith('.schema.json'):
                schema_name = filename[:-12]  # Remove .schema.json
            elif filename.endswith('.json'):
                schema_name = filename[:-5]  # Remove .json
            else:
                schema_name = filename
            
            # Convert to component name format
            ref_component = re.sub(r'([a-z0-9])([A-Z])', r'\1-\2', schema_name).lower()
            
            # Determine directory context from the path
            parent_dir = ref_path.parent.name if ref_path.parent.name != '.' else None
            
            if parent_dir == '_base':
                # Base schemas: if already starts with 'base-', don't add another prefix
                if ref_component.startswith('base-'):
                    return ref_component
                else:
                    return f"base-{ref_component}"
            elif parent_dir and parent_dir != '.':
                # Other directory schemas get directory prefix
                return f"{parent_dir}-{ref_component}"
            else:
                # Same directory or no directory context
                if current_component.startswith('base-'):
                    return f"base-{ref_component}"
                elif '-' in current_component:
                    current_dir = current_component.split('-')[0]
                    return f"{current_dir}-{ref_component}"
                else:
                    return ref_component
        
        # Handle URL-based refs
        if ref.startswith('http'):
            parsed = urlparse(ref)
            path_parts = parsed.path.strip('/').split('/')
            if path_parts and path_parts[-1].endswith('.schema.json'):
                schema_name = path_parts[-1][:-12]  # Remove .schema.json
                return re.sub(r'([a-z0-9])([A-Z])', r'\1-\2', schema_name).lower()
        
        return None
    
    def analyze_dependencies(self):
        """Analyze all schemas to build dependency graph."""
        print("\nAnalyzing dependencies...")
        
        for component_name, schema in self.schemas.items():
            deps = self.extract_ref_dependencies(schema, component_name)
            self.dependencies[component_name] = deps
            
            if deps:
                print(f"  {component_name} depends on: {', '.join(sorted(deps))}")
            else:
                print(f"  {component_name} has no dependencies")
    
    def get_schema_category(self, file_path: str) -> str:
        """Determine the category/type of a schema based on its path."""
        if '_base/' in file_path:
            return 'foundation'
        elif 'components/' in file_path:
            return 'component'
        elif 'entities/' in file_path:
            return 'entity'
        elif 'snippets/' in file_path:
            return 'snippet'
        elif 'templates/' in file_path:
            return 'template'
        elif 'api/' in file_path:
            return 'api'
        elif 'events/' in file_path:
            return 'event'
        elif 'laws/' in file_path:
            return 'law'
        elif 'payloads/' in file_path:
            return 'payload'
        elif 'tables/' in file_path:
            return 'table'
        else:
            return 'schema'
    
    def get_system_info(self, category: str) -> Dict[str, Any]:
        """Get system, owner, and dependency info based on schema category."""
        return self.system_mapping.get(category, self.system_mapping['default'])
    
    def get_lifecycle(self, schema: Dict) -> str:
        """Determine the lifecycle stage of a schema based on its content."""
        # Look for a lifecycle hint in the schema itself, e.g., "lifecycle": "experimental"
        lifecycle = schema.get('lifecycle', schema.get('status', 'production')).lower()
        
        if lifecycle in ['experimental', 'dev', 'development']:
            return 'experimental'
        if lifecycle == 'deprecated':
            return 'deprecated'
        
        # Default to production if no other clear stage is specified
        return 'production'
    
    def is_production_schema(self, file_path: str) -> bool:
        """Check if this schema is one of the 23 production-ready schemas for code generation."""
        filename = Path(file_path).name
        return filename in self.production_schemas
    
    def get_tags_for_schema(self, schema: Dict, file_path: str) -> List[str]:
        """Generate appropriate tags for a schema."""
        tags = ['schema', 'auto-generated']
        
        # Add category-based tags
        category = self.get_schema_category(file_path)
        tags.append(category)
        
        # Special tagging for production schemas
        if self.is_production_schema(file_path):
            tags.extend(['production-ready', 'code-generation', 'rust-target'])
        else:
            tags.append('assembly-schema')
        
        # Add tags based on schema content
        title = schema.get('title', '').lower()
        description = schema.get('description', '').lower()
        
        if any(word in title + description for word in ['physics', 'quantum', 'energy']):
            tags.append('physics')
        if any(word in title + description for word in ['cognitive', 'memory', 'perspective']):
            tags.append('cognitive')
        if any(word in title + description for word in ['tenant', 'user', 'member']):
            tags.append('tenant')
        if any(word in title + description for word in ['component', 'ecs']):
            tags.append('ecs')
        if any(word in title + description for word in ['thread', 'entity']):
            tags.append('entity')
        if any(word in title + description for word in ['base', 'foundation', 'shared']):
            tags.append('foundation')
        
        return sorted(list(set(tags)))
    
    def get_subcomponent_relationships(self, component_name: str, schema: Dict) -> Optional[str]:
        """Determine if this schema should be a subcomponent of another."""
        file_path = self.schema_files[component_name]
        
        # Rule 1: Non-base schemas that extend base schemas should be subcomponents
        if not component_name.startswith('base-'):
            deps = self.dependencies.get(component_name, set())
            
            # Find the most specific base dependency, but prioritize direct inheritance
            base_deps = [dep for dep in deps if dep.startswith('base-')]
            if base_deps:
                # For components, prefer base-component if present
                if 'components/' in file_path and 'base-component' in base_deps:
                    return 'component:default/base-component'
                # For entities, prefer base-entity or more specific entity bases
                elif 'entities/' in file_path:
                    entity_bases = [dep for dep in base_deps if 'entity' in dep]
                    if entity_bases:
                        # Sort by specificity (longer names are more specific)
                        entity_bases.sort(key=len, reverse=True)
                        return f'component:default/{entity_bases[0]}'
                # Otherwise, use the most specific base
                base_deps.sort(key=len, reverse=True)
                return f'component:default/{base_deps[0]}'
        
        # Rule 2: Entity schemas that inherit from other entity schemas
        if 'entities/' in file_path and not component_name.startswith('base-'):
            # Check for inheritance patterns in allOf
            all_of = schema.get('allOf', [])
            for item in all_of:
                if '$ref' in item:
                    ref_comp = self.ref_to_component_name(item['$ref'], component_name)
                    if ref_comp and ref_comp != component_name and ref_comp in self.schemas:
                        return f'component:default/{ref_comp}'
        
        # Rule 3: Component schemas that extend BaseComponent
        if 'components/' in file_path and not component_name.startswith('base-'):
            deps = self.dependencies.get(component_name, set())
            if 'base-component' in deps:
                return 'component:default/base-component'
        
        return None

    def generate_core_architecture_components(self) -> List[Dict]:
        """Generates the static, high-level architecture components for the catalog."""
        core_components = [
            # ==========================================================================
            # Ownership: Teams (kind: Group)
            # ==========================================================================
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "Group",
                "metadata": {
                    "name": "team-system-architecture", "title": "System Architecture Team",
                    "description": "Owns the overall system design and foundational principles.",
                    "members": ["user:default/phaiel"]
                },
                "spec": {"type": "team", "children": []}
            },
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "Group",
                "metadata": {
                    "name": "team-platform-infrastructure", "title": "Platform Infrastructure Team",
                    "description": "Owns the API gateway, databases, and deployment infrastructure.",
                    "members": ["user:default/phaiel"]
                },
                "spec": {"type": "team", "children": []}
            },
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "Group",
                "metadata": {
                    "name": "team-cognitive-modeling", "title": "Cognitive Modeling Team",
                    "description": "Owns the agentic framework and high-level cognitive analysis.",
                    "members": ["user:default/phaiel"]
                },
                "spec": {"type": "team", "children": []}
            },
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "Group",
                "metadata": {
                    "name": "team-physics-core", "title": "Physics Core Team",
                    "description": "Owns the implementation of the quantum and classical physics engines.",
                    "members": ["user:default/phaiel"]
                },
                "spec": {"type": "team", "children": []}
            },

            # ==========================================================================
            # Domains & Systems
            # ==========================================================================
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "Domain",
                "metadata": {
                    "name": "cognitive-systems",
                    "description": "Systems related to cognitive modeling, simulation, and analysis."
                },
                "spec": {"owner": "team-system-architecture"}
            },
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "System",
                "metadata": {
                    "name": "familiar-physics-engine", "title": "Familiar Cognitive Physics Engine",
                    "description": "A quantum-classical hybrid system for simulating cognitive processes, memory, and relationships.",
                },
                "spec": {"owner": "team-system-architecture", "domain": "cognitive-systems"}
            },
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "System",
                "metadata": {
                    "name": "familiar-schema-system", "title": "Familiar Schema System",
                    "description": "The foundational system for defining, generating, and managing all data schemas (the 'nouns')."
                },
                "spec": {"owner": "team-system-architecture", "domain": "cognitive-systems"}
            },

            # ==========================================================================
            # Components: Core Services, Libraries, and Workflows
            # ==========================================================================
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "Component",
                "metadata": {
                    "name": "physics-engine-core", "title": "Core Physics Engine Library",
                    "description": "The central library containing the ECS world, physics laws, and simulation logic. It is not a standalone service but the heart of other services."
                },
                "spec": {
                    "type": "library", "lifecycle": "production", "owner": "team-physics-core",
                    "system": "familiar-physics-engine"
                }
            },
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "Component",
                "metadata": {
                    "name": "agentic-ingestion-service", "title": "Agentic Ingestion Service (The Loom)",
                    "description": "A Windmill-orchestrated service that consumes user 'weaves' and uses an agentic pipeline (The Heddle) to create cognitive entities."
                },
                "spec": {
                    "type": "service", "lifecycle": "production", "owner": "team-cognitive-modeling",
                    "system": "familiar-physics-engine", "subcomponentOf": "component:default/physics-engine-core",
                    "consumesApis": ["api:default/quantum-classical-handoff-api"],
                    "dependsOn": ["resource:default/redpanda-cluster", "resource:default/timescaledb-physics-db", "component:default/quantum-physics-service"]
                }
            },
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "Component",
                "metadata": {
                    "name": "cognitive-analysis-engine", "title": "Cognitive Analysis Engine",
                    "description": "Provides the 5-level cognitive hierarchy for querying the memory manifold and generating scientifically-grounded insights."
                },
                "spec": {
                    "type": "service", "lifecycle": "production", "owner": "team-cognitive-modeling",
                    "system": "familiar-physics-engine", "subcomponentOf": "component:default/physics-engine-core",
                    "providesApis": ["api:default/graphql-api"],
                    "dependsOn": ["resource:default/timescaledb-physics-db"]
                }
            },
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "Component",
                "metadata": {
                    "name": "quantum-physics-service", "title": "Quantum Physics Service (QuTiP)",
                    "description": "A Python-based service responsible for all quantum calculations, including superposition, coherence, and collapse, using the QuTiP library."
                },
                "spec": {
                    "type": "service", "lifecycle": "production", "owner": "team-physics-core",
                    "system": "familiar-physics-engine", "subcomponentOf": "component:default/physics-engine-core",
                    "consumesApis": ["api:default/quantum-classical-handoff-api"],
                    "dependsOn": ["resource:default/redpanda-cluster"]
                }
            },
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "Component",
                "metadata": {
                    "name": "classical-physics-service", "title": "Classical Physics Service (Particular)",
                    "description": "A Rust-based service responsible for all classical N-body physics simulations, including bond tension and energy dynamics, using the Particular library."
                },
                "spec": {
                    "type": "service", "lifecycle": "production", "owner": "team-physics-core",
                    "system": "familiar-physics-engine", "subcomponentOf": "component:default/physics-engine-core",
                    "consumesApis": ["api:default/quantum-classical-handoff-api"],
                    "dependsOn": ["resource:default/redpanda-cluster"]
                }
            },
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "Component",
                "metadata": {
                    "name": "physics-api-gateway", "title": "Physics API Gateway",
                    "description": "The primary user-facing event gateway. Translates user actions into Redpanda events and streams real-time updates."
                },
                "spec": {
                    "type": "service", "lifecycle": "production", "owner": "team-platform-infrastructure",
                    "system": "familiar-physics-engine",
                    "providesApis": ["api:default/physics-engine-public-api"],
                    "dependsOn": ["resource:default/redpanda-cluster"]
                }
            },
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "Component",
                "metadata": {
                    "name": "cognitive-management-ui", "title": "Cognitive Management UI",
                    "description": "User-facing web application for managing ground-truth entities like Threads and Bonds."
                },
                "spec": {
                    "type": "website", "lifecycle": "production", "owner": "team-platform-infrastructure",
                    "system": "familiar-physics-engine",
                    "consumesApis": ["api:default/physics-engine-public-api", "api:default/graphql-api"],
                    "dependsOn": ["component:default/physics-api-gateway"]
                }
            },
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "Component",
                "metadata": {
                    "name": "dag-deep-cognitive-synthesis", "title": "DAG: Deep Cognitive Synthesis",
                    "description": "The core Windmill workflow for answering complex user queries with a <10s SLA."
                },
                "spec": {
                    "type": "service", "lifecycle": "production", "owner": "team-cognitive-modeling",
                    "system": "familiar-physics-engine", "subcomponentOf": "component:default/agentic-ingestion-service"
                }
            },
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "Component",
                "metadata": {
                    "name": "dag-memory-consolidation", "title": "DAG: Memory Consolidation",
                    "description": "The daily background Windmill workflow for consolidating memories and evolving Motifs."
                },
                "spec": {
                    "type": "service", "lifecycle": "production", "owner": "team-physics-core",
                    "system": "familiar-physics-engine", "subcomponentOf": "component:default/physics-engine-core"
                }
            },
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "Component",
                "metadata": {"name": "schema-assembly-pipeline", "title": "Schema Assembly Pipeline"},
                "spec": {
                    "type": "library", "lifecycle": "production", "owner": "team-platform-infrastructure",
                    "system": "familiar-schema-system"
                }
            },

            # ==========================================================================
            # APIs: The Contracts
            # ==========================================================================
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "API",
                "metadata": {
                    "name": "physics-engine-public-api", "title": "Familiar Physics Public API",
                    "description": "The public-facing REST/event API for submitting data and receiving real-time updates."
                },
                "spec": {
                    "type": "openapi", "lifecycle": "production", "owner": "team-platform-infrastructure",
                    "system": "familiar-physics-engine",
                    "definition": "$text: ./docs/v3/schemas/api/README.md"
                }
            },
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "API",
                "metadata": {
                    "name": "graphql-api", "title": "Cognitive Manifold GraphQL API",
                    "description": "The GraphQL API for complex, physics-aware queries of the cognitive manifold."
                },
                "spec": {
                    "type": "graphql", "lifecycle": "production", "owner": "team-cognitive-modeling",
                    "system": "familiar-physics-engine",
                    "definition": "$text: ./integration/database_data_management.md#complete-graphql-api-for-physics-integration"
                }
            },
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "API",
                "metadata": {
                    "name": "quantum-classical-handoff-api", "title": "Quantum-Classical Handoff API",
                    "description": "The internal event-based API for managing state transitions between the quantum and classical engines, defined by the CollapsePayload schema."
                },
                "spec": {
                    "type": "asyncapi", "lifecycle": "production", "owner": "team-physics-core",
                    "system": "familiar-physics-engine",
                    "definition": "$text: ./docs/v3/schemas/payloads/collapse_payload.schema.json"
                }
            },

            # ==========================================================================
            # Resources: The Infrastructure
            # ==========================================================================
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "Resource",
                "metadata": {
                    "name": "timescaledb-physics-db", "title": "TimescaleDB Physics Database",
                    "description": "The primary time-series vector database for storing all immutable entity versions and mutable physics states."
                },
                "spec": {"type": "database", "owner": "team-platform-infrastructure", "system": "familiar-physics-engine"}
            },
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "Resource",
                "metadata": {
                    "name": "redpanda-cluster", "title": "Redpanda Streaming Cluster",
                    "description": "The event streaming platform that enables asynchronous, event-driven communication between all system components."
                },
                "spec": {"type": "messaging-queue", "owner": "team-platform-infrastructure", "system": "familiar-physics-engine"}
            },
            {
                "apiVersion": "backstage.io/v1alpha1", "kind": "Resource",
                "metadata": {
                    "name": "monitoring-stack", "title": "Monitoring & Observability Stack",
                    "description": "The Prometheus, Grafana, and Jaeger stack for monitoring the health and performance of the physics engine."
                },
                "spec": {"type": "monitoring", "owner": "team-platform-infrastructure", "system": "familiar-physics-engine"}
            }
        ]
        return core_components

    def generate_component_entry(self, component_name: str) -> Dict:
        """Generate a single Backstage component entry for a schema."""
        schema = self.schemas[component_name]
        file_path = self.schema_files[component_name]
        repo_url = "https://github.com/phaiel/familiar"

        # Basic spec
        spec = {
            "type": "schema",
            "lifecycle": self.get_lifecycle(schema),
        }

        # Add system, owner, and dependencies from mapping
        category = self.get_schema_category(file_path)
        system_info = self.get_system_info(category)
        spec['owner'] = system_info['owner']
        spec['system'] = system_info['system']

        # Combine dependencies from $refs and system mapping
        internal_deps = self.dependencies.get(component_name, set())
        formatted_internal_deps = [f"component:default/{dep}" for dep in sorted(internal_deps)]
        
        external_deps = system_info.get('dependsOn', [])
        
        all_deps = sorted(list(set(formatted_internal_deps + external_deps)))
        if all_deps:
            spec['dependsOn'] = all_deps

        # Subcomponent relationships
        subcomponent_of = self.get_subcomponent_relationships(component_name, schema)
        if subcomponent_of:
            spec['subcomponentOf'] = subcomponent_of
            
        # Part-of relationships (for domain/system grouping)
        # This is an example; adjust as needed
        part_of = []
        if 'entities' in file_path or 'components' in file_path:
            part_of.append('domain:default/cognitive-systems')
        
        if part_of:
            spec['partOf'] = part_of

        # Annotations
        annotations = {
            "backstage.io/managed-by-location": f"url:{repo_url}/blob/main/docs/v3/scripts/generate_catalog_info.py",
            "backstage.io/source-location": f"url:{repo_url}/blob/main/{file_path}",
            "backstage.io/techdocs-ref": f"url:{repo_url}/blob/main/{file_path}",
            "familiar.ai/schema-category": category,
            "familiar.ai/schema-title": schema.get("title", "N/A"),
        }
        
        # Add special annotations for production schemas
        if self.is_production_schema(file_path):
            annotations.update({
                "familiar.ai/production-schema": "true",
                "familiar.ai/code-generation-target": "rust",
                "familiar.ai/backstage-action": "scaffold-rust-struct",
                "familiar.ai/assembled-schema-path": f"docs/v3/schemas/assembled/{Path(file_path).name}"
            })
        else:
            annotations["familiar.ai/production-schema"] = "false"

        # Links
        links = [
            {
                "title": "View Schema Source",
                "url": f"{repo_url}/blob/main/{file_path}",
                "icon": "source"
            },
            {
                "title": "View in JSON Crack",
                "url": f"https://jsoncrack.com/editor?json={repo_url}/blob/main/{file_path}",
                "icon": "visibility"
            }
        ]
        
        # Add special links for production schemas
        if self.is_production_schema(file_path):
            links.extend([
                {
                    "title": "Generate Rust Struct",
                    "url": f"{repo_url}/blob/main/docs/v3/templates/rust-entity/",
                    "icon": "code",
                    "type": "backstage-action"
                },
                {
                    "title": "View Assembled Schema",
                    "url": f"{repo_url}/blob/main/docs/v3/schemas/assembled/{Path(file_path).name}",
                    "icon": "build"
                }
            ])

        # Final Entry
        entry = {
            "apiVersion": "backstage.io/v1alpha1",
            "kind": "Component",
            "metadata": {
                "name": component_name,
                "title": schema.get("title", component_name.replace('-', ' ').title()),
                "description": schema.get("description", "No description provided."),
                "tags": self.get_tags_for_schema(schema, file_path),
                "annotations": annotations,
                "links": links
            },
            "spec": spec
        }
        
        return entry

    def generate_catalog_info(self) -> List[Dict]:
        """Generate a list of all catalog component entries."""
        self.load_all_schemas()
        self.analyze_dependencies()
        
        print("\nGenerating catalog entries...")
        
        all_components = self.generate_core_architecture_components()
        
        print(f"Generated {len(all_components)} core architecture components.")
        print("\nGenerating schema component entries...")

        for component_name in sorted(self.schemas.keys()):
            entry = self.generate_component_entry(component_name)
            all_components.append(entry)
            
            print(f"  Generated entry for: {component_name}")
            
        return all_components

    def write_catalog_info(self, output_file: str = "catalog-info.yaml"):
        """Generate and write the complete catalog to a YAML file."""
        catalog_items = self.generate_catalog_info()
        
        # Resolve output path relative to the project root for robustness
        output_path = self.schemas_dir.parent.parent.parent / output_file
        
        print(f"\nWriting {len(catalog_items)} components to {output_path}...")
        try:
            # Use dump_all for a multi-document YAML file
            with open(output_path, 'w') as f:
                yaml.dump_all(catalog_items, f, sort_keys=False, default_flow_style=False)
            print(f"Successfully wrote catalog file.")
        except IOError as e:
            print(f"Error writing to {output_path}: {e}")

    def run(self, output_file: str):
        """Execute the full catalog generation process."""
        print("=== Familiar Schema Catalog Generator ===")
        print(f"  Schemas directory: {self.schemas_dir}")
        
        self.load_all_schemas()
        self.analyze_dependencies()
        self.write_catalog_info(output_file)
        
        print("\n=======================================")

def main():
    """Main function to run the script from the command line."""
    # The project root is three levels up from this script (scripts/v3/docs/familiar)
    project_root = Path(__file__).parent.parent.parent.parent
    
    parser = argparse.ArgumentParser(description="Generate Backstage catalog-info.yaml from JSON schemas.")
    parser.add_argument(
        "--schemas-dir",
        type=str,
        default=str(project_root / "docs/v3/schemas"),
        help="Path to the root schemas directory."
    )
    parser.add_argument(
        "--output",
        type=str,
        default="catalog-info.yaml",
        help="Output file name for the Backstage catalog (e.g., catalog-info.yaml)."
    )
    
    args = parser.parse_args()

    # Instantiate the generator with the absolute path to the schemas directory
    generator = SchemaCatalogGenerator(schemas_dir=Path(args.schemas_dir))
    generator.run(output_file=args.output)

if __name__ == "__main__":
    main() 
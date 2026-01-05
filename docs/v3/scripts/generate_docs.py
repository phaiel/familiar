#!/usr/bin/env python3
"""
Familiar Schema Documentation Generator
Auto-generates TechDocs documentation from JSON schemas.
"""

import json
import yaml
from pathlib import Path
from typing import Dict
from datetime import datetime
import argparse


class SchemaDocGenerator:
    """Generates documentation from JSON schemas."""
    
    def __init__(self, schemas_dir: Path, docs_dir: Path):
        self.schemas_dir = schemas_dir.resolve()
        self.docs_dir = docs_dir.resolve()
        self.schemas: Dict[str, Dict] = {}
        
    def load_all_schemas(self):
        """Load all JSON schema files."""
        print("Loading schemas...")
        
        for schema_file in self.schemas_dir.rglob("*.json"):
            try:
                with open(schema_file, 'r') as f:
                    schema = json.load(f)
                
                # Use relative path as key
                rel_path = schema_file.relative_to(self.schemas_dir)
                self.schemas[str(rel_path)] = schema
                print(f"  Loaded: {rel_path}")
                
            except Exception as e:
                print(f"  Error loading {schema_file}: {e}")
        
        print(f"Loaded {len(self.schemas)} schemas")
    
    def get_friendly_title(self, schema: Dict, schema_path: str) -> str:
        """Generate a friendly title from schema or file path."""
        # If schema has a title, use it
        if 'title' in schema:
            return schema['title']
        
        # Extract filename without extension
        path_obj = Path(schema_path)
        filename = path_obj.stem
        
        # Remove .schema suffix if present
        if filename.endswith('.schema'):
            filename = filename[:-7]
        
        # Convert CamelCase to Title Case with spaces
        # ThreadType -> Thread Type
        # FeatureFlagMap -> Feature Flag Map
        import re
        title = re.sub(r'([a-z0-9])([A-Z])', r'\1 \2', filename)
        
        # Handle special cases for snippets
        if 'snippets' in schema_path:
            if 'fields' in schema_path:
                title += ' Field'
            elif 'types' in schema_path:
                title += ' Type'
        
        return title
    
    def generate_docs(self):
        """Generate documentation."""
        print("\nGenerating documentation...")
        
        # Create docs directory  
        docs_root = self.docs_dir / 'docs'
        docs_root.mkdir(exist_ok=True)
        
        # Group schemas by category
        categories = {}
        for schema_path, schema in self.schemas.items():
            if '_base' in schema_path:
                category = 'Foundation'
            elif 'components' in schema_path:
                category = 'Components'
            elif 'entities' in schema_path:
                category = 'Entities'
            elif 'snippets' in schema_path:
                category = 'Snippets'
            elif 'events' in schema_path:
                category = 'Events'
            elif 'tables' in schema_path:
                category = 'Tables'
            elif 'payloads' in schema_path:
                category = 'Payloads'
            elif 'workflows' in schema_path:
                category = 'Workflows'
            elif 'laws' in schema_path:
                category = 'Laws'
            elif 'api' in schema_path:
                category = 'API'
            else:
                category = 'Other'
            
            if category not in categories:
                categories[category] = []
            categories[category].append((schema_path, schema))
        
        # Generate main index
        home_content = f"""# Familiar Schema System

Auto-generated documentation for the Familiar Schema System.

*Generated on {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}*

## Overview

The Familiar Schema System contains **{len(self.schemas)} schemas** organized across {len(categories)} categories:

"""
        
        for category, schemas in sorted(categories.items()):
            home_content += f"\n### {category} ({len(schemas)} schemas)\n\n"
            for schema_path, schema in sorted(schemas):
                title = self.get_friendly_title(schema, schema_path)
                description = schema.get('description', 'No description available')
                # Truncate long descriptions
                if len(description) > 100:
                    description = description[:97] + "..."
                home_content += f"- **{title}**: {description}\n"
        
        home_content += f"""

## Architecture

The schemas follow a hierarchical structure:

- **Foundation Schemas** (`_base/`): Core base types and shared definitions
- **Component Schemas** (`components/`): ECS components for modular functionality  
- **Entity Schemas** (`entities/`): Complete entity definitions combining components
- **Snippet Schemas** (`snippets/`): Reusable field and type definitions

## Pipeline Integration

This documentation is automatically generated as part of the schema pipeline:

```bash
cd docs/v3
make generate-docs  # Generate this documentation
make all           # Run complete pipeline including docs
```

## Backstage Integration

The schemas are also integrated with Backstage for catalog management and visualization at http://localhost:3000.
"""
        
        with open(docs_root / 'index.md', 'w') as f:
            f.write(home_content)
        
        # Generate MkDocs config
        config = {
            'site_name': 'Familiar Schema System',
            'site_description': 'Auto-generated schema documentation',
            'nav': [
                {'Home': 'index.md'}
            ],
            'plugins': ['techdocs-core'],
            'theme': {
                'name': 'material',
                'palette': {
                    'primary': 'blue',
                    'accent': 'blue'
                },
                'features': [
                    'navigation.top',
                    'search.highlight'
                ]
            }
        }
        
        with open(self.docs_dir / 'mkdocs.yml', 'w') as f:
            yaml.dump(config, f, default_flow_style=False)
        
        print(f"✅ Generated documentation in {docs_root}")
        print(f"✅ Generated MkDocs config: {self.docs_dir}/mkdocs.yml")
    
    def run(self):
        """Run the complete documentation generation."""
        print("=== Familiar Schema Documentation Generator ===")
        self.load_all_schemas()
        self.generate_docs()
        print("=== Documentation Generation Complete ===")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Familiar Schema Documentation Generator.")
    parser.add_argument(
        "--schemas-dir",
        type=Path,
        required=True,
        help="Absolute path to the source schemas directory."
    )
    parser.add_argument(
        "--docs-dir",
        type=Path,
        required=True,
        help="Absolute path to the root docs directory for output."
    )
    args = parser.parse_args()
    
    generator = SchemaDocGenerator(schemas_dir=args.schemas_dir, docs_dir=args.docs_dir)
    generator.run()

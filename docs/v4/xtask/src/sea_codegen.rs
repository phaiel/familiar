//! SeaORM Entity Code Generator
//!
//! Generates SeaORM entity files from database schemas (x-familiar-kind: "database").
//! Uses schema extensions to determine table names, relations, and field attributes.

use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Schema extensions for SeaORM
#[derive(Debug, Clone, Deserialize)]
pub struct SeaOrmExtensions {
    #[serde(rename = "x-familiar-table")]
    pub table: String,
    #[serde(rename = "x-familiar-module")]
    pub module: String,
    #[serde(rename = "x-familiar-primary-key")]
    pub primary_key: PrimaryKey,
    #[serde(rename = "x-familiar-relations", default)]
    pub relations: Vec<Relation>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum PrimaryKey {
    Single(String),
    Composite(Vec<String>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Relation {
    #[serde(rename = "type")]
    pub rel_type: String,
    pub target: String,
    #[serde(default)]
    pub field: Option<String>,
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
}

/// Field metadata extracted from schema
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub rust_type: String,
    pub sea_orm_attrs: Vec<String>,
    pub is_primary_key: bool,
    pub is_optional: bool,
}

/// Generated enum for SeaORM DeriveActiveEnum
#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub rust_name: String,
    pub db_value: String,
}

/// Result of parsing a database schema
#[derive(Debug)]
pub struct ParsedEntity {
    pub schema_name: String,
    pub entity_name: String,
    pub table_name: String,
    pub module: String,
    pub fields: Vec<FieldInfo>,
    pub enums: Vec<EnumInfo>,
    pub relations: Vec<Relation>,
    pub imports: HashSet<String>,
}

/// Type mapping from JSON Schema to SeaORM
pub struct TypeMapper {
    /// Maps primitive schema refs to Rust types
    primitive_map: HashMap<String, (String, String)>, // (rust_type, import)
}

impl TypeMapper {
    pub fn new() -> Self {
        let mut primitive_map = HashMap::new();
        
        // Map familiar primitives to Rust types
        primitive_map.insert("UserId".to_string(), ("UserId".to_string(), "familiar_primitives::UserId".to_string()));
        primitive_map.insert("TenantId".to_string(), ("TenantId".to_string(), "familiar_primitives::TenantId".to_string()));
        primitive_map.insert("SessionId".to_string(), ("SessionId".to_string(), "familiar_primitives::SessionId".to_string()));
        primitive_map.insert("ChannelId".to_string(), ("ChannelId".to_string(), "familiar_primitives::ChannelId".to_string()));
        primitive_map.insert("MessageId".to_string(), ("MessageId".to_string(), "familiar_primitives::MessageId".to_string()));
        primitive_map.insert("EntityId".to_string(), ("EntityId".to_string(), "familiar_primitives::EntityId".to_string()));
        primitive_map.insert("ThreadId".to_string(), ("ThreadId".to_string(), "familiar_primitives::ThreadId".to_string()));
        primitive_map.insert("MagicLinkId".to_string(), ("MagicLinkId".to_string(), "familiar_primitives::MagicLinkId".to_string()));
        primitive_map.insert("InvitationId".to_string(), ("InvitationId".to_string(), "familiar_primitives::InvitationId".to_string()));
        primitive_map.insert("JoinRequestId".to_string(), ("JoinRequestId".to_string(), "familiar_primitives::JoinRequestId".to_string()));
        primitive_map.insert("ConsentRecordId".to_string(), ("ConsentRecordId".to_string(), "familiar_primitives::ConsentRecordId".to_string()));
        primitive_map.insert("AuditLogId".to_string(), ("AuditLogId".to_string(), "familiar_primitives::AuditLogId".to_string()));
        primitive_map.insert("Email".to_string(), ("String".to_string(), "".to_string())); // Email stored as String in DB
        primitive_map.insert("PasswordHash".to_string(), ("String".to_string(), "".to_string()));
        primitive_map.insert("InviteCode".to_string(), ("String".to_string(), "".to_string()));
        primitive_map.insert("InviteRole".to_string(), ("InviteRole".to_string(), "".to_string())); // Enum generated separately
        
        Self { primitive_map }
    }

    /// Map a JSON Schema property to a Rust type
    pub fn map_property(&self, name: &str, prop: &Value, required: bool, imports: &mut HashSet<String>) -> FieldInfo {
        let (rust_type, sea_orm_attrs, is_optional) = self.resolve_type(prop, required, imports);
        
        FieldInfo {
            name: name.to_string(),
            rust_type,
            sea_orm_attrs,
            is_primary_key: false,
            is_optional,
        }
    }

    fn resolve_type(&self, prop: &Value, required: bool, imports: &mut HashSet<String>) -> (String, Vec<String>, bool) {
        // Check for x-familiar-column-type override
        if let Some(col_type) = prop.get("x-familiar-column-type").and_then(|v| v.as_str()) {
            return self.handle_column_type_override(col_type, required, imports);
        }

        // Check for $ref to primitive
        if let Some(ref_path) = prop.get("$ref").and_then(|v| v.as_str()) {
            return self.handle_ref(ref_path, required, imports);
        }

        // Check for anyOf (nullable ref)
        if let Some(any_of) = prop.get("anyOf").and_then(|v| v.as_array()) {
            return self.handle_any_of(any_of, imports);
        }

        // Handle type arrays (nullable types)
        if let Some(types) = prop.get("type").and_then(|v| v.as_array()) {
            return self.handle_type_array(types, prop, imports);
        }

        // Handle single type
        if let Some(type_str) = prop.get("type").and_then(|v| v.as_str()) {
            return self.handle_single_type(type_str, prop, required, imports);
        }

        // Fallback for `true` schema (any type) - treat as Json
        if prop.is_boolean() && prop.as_bool() == Some(true) {
            return ("Json".to_string(), vec!["column_type = \"JsonBinary\"".to_string()], false);
        }

        // Unknown type - use Json as fallback
        ("Json".to_string(), vec!["column_type = \"JsonBinary\"".to_string()], false)
    }

    fn handle_column_type_override(&self, col_type: &str, required: bool, _imports: &mut HashSet<String>) -> (String, Vec<String>, bool) {
        match col_type {
            "JsonBinary" => {
                let attrs = vec!["column_type = \"JsonBinary\"".to_string()];
                if required {
                    ("Json".to_string(), attrs, false)
                } else {
                    ("Option<Json>".to_string(), vec!["column_type = \"JsonBinary\"".to_string(), "nullable".to_string()], true)
                }
            }
            _ => ("Json".to_string(), vec!["column_type = \"JsonBinary\"".to_string()], false)
        }
    }

    fn handle_ref(&self, ref_path: &str, required: bool, imports: &mut HashSet<String>) -> (String, Vec<String>, bool) {
        // Extract type name from path like "../primitives/UserId.schema.json"
        let type_name = ref_path
            .split('/')
            .last()
            .unwrap_or("")
            .trim_end_matches(".schema.json");
        
        if let Some((rust_type, import)) = self.primitive_map.get(type_name) {
            if !import.is_empty() {
                imports.insert(import.clone());
            }
            let attrs = Vec::new();
            if required {
                (rust_type.clone(), attrs, false)
            } else {
                (format!("Option<{}>", rust_type), vec!["nullable".to_string()], true)
            }
        } else {
            // Check for known enum types that need inline definition
            // These are defined in external schemas but need SeaORM DeriveActiveEnum
            let attrs = Vec::new();
            if required {
                (type_name.to_string(), attrs, false)
            } else {
                (format!("Option<{}>", type_name), vec!["nullable".to_string()], true)
            }
        }
    }

    fn handle_any_of(&self, any_of: &[Value], imports: &mut HashSet<String>) -> (String, Vec<String>, bool) {
        // anyOf with null means optional
        let non_null: Vec<_> = any_of.iter()
            .filter(|v| v.get("type").and_then(|t| t.as_str()) != Some("null"))
            .collect();
        
        if non_null.len() == 1 {
            let (rust_type, mut attrs, _) = self.resolve_type(non_null[0], true, imports);
            attrs.push("nullable".to_string());
            (format!("Option<{}>", rust_type), attrs, true)
        } else {
            // Complex anyOf - use Json
            ("Option<Json>".to_string(), vec!["column_type = \"JsonBinary\"".to_string(), "nullable".to_string()], true)
        }
    }

    fn handle_type_array(&self, types: &[Value], prop: &Value, imports: &mut HashSet<String>) -> (String, Vec<String>, bool) {
        let type_strs: Vec<&str> = types.iter()
            .filter_map(|v| v.as_str())
            .collect();
        
        let has_null = type_strs.contains(&"null");
        let non_null: Vec<&&str> = type_strs.iter().filter(|&&t| t != "null").collect();
        
        if non_null.len() == 1 {
            let (rust_type, mut attrs, _) = self.map_simple_type(non_null[0], prop, imports);
            if has_null {
                attrs.push("nullable".to_string());
                (format!("Option<{}>", rust_type), attrs, true)
            } else {
                (rust_type, attrs, false)
            }
        } else {
            // Multiple non-null types - use Json
            let attrs = vec!["column_type = \"JsonBinary\"".to_string()];
            if has_null {
                ("Option<Json>".to_string(), vec!["column_type = \"JsonBinary\"".to_string(), "nullable".to_string()], true)
            } else {
                ("Json".to_string(), attrs, false)
            }
        }
    }

    fn handle_single_type(&self, type_str: &str, prop: &Value, required: bool, imports: &mut HashSet<String>) -> (String, Vec<String>, bool) {
        let (rust_type, attrs, _) = self.map_simple_type(type_str, prop, imports);
        if required {
            (rust_type, attrs, false)
        } else {
            (format!("Option<{}>", rust_type), vec!["nullable".to_string()], true)
        }
    }

    fn map_simple_type(&self, type_str: &str, prop: &Value, _imports: &mut HashSet<String>) -> (String, Vec<String>, bool) {
        match type_str {
            "string" => {
                if let Some(format) = prop.get("format").and_then(|v| v.as_str()) {
                    match format {
                        "date-time" => ("DateTimeUtc".to_string(), Vec::new(), false),
                        "uuid" => ("Uuid".to_string(), Vec::new(), false),
                        _ => ("String".to_string(), Vec::new(), false),
                    }
                } else {
                    ("String".to_string(), Vec::new(), false)
                }
            }
            "integer" => {
                if let Some(format) = prop.get("format").and_then(|v| v.as_str()) {
                    match format {
                        "int64" => ("i64".to_string(), Vec::new(), false),
                        "int32" => ("i32".to_string(), Vec::new(), false),
                        _ => ("i32".to_string(), Vec::new(), false),
                    }
                } else {
                    ("i32".to_string(), Vec::new(), false)
                }
            }
            "number" => {
                if let Some(format) = prop.get("format").and_then(|v| v.as_str()) {
                    match format {
                        "double" | "float64" => ("f64".to_string(), Vec::new(), false),
                        "float" | "float32" => ("f32".to_string(), Vec::new(), false),
                        _ => ("f64".to_string(), Vec::new(), false),
                    }
                } else {
                    ("f64".to_string(), Vec::new(), false)
                }
            }
            "boolean" => ("bool".to_string(), Vec::new(), false),
            "object" | "array" => ("Json".to_string(), vec!["column_type = \"JsonBinary\"".to_string()], false),
            _ => ("String".to_string(), Vec::new(), false),
        }
    }
}

/// Parse a database schema file into entity metadata
pub fn parse_schema(schema_path: &Path) -> anyhow::Result<ParsedEntity> {
    let content = fs::read_to_string(schema_path)?;
    let schema: Value = serde_json::from_str(&content)?;
    
    // Extract SeaORM extensions
    let extensions: SeaOrmExtensions = serde_json::from_value(schema.clone())?;
    
    // Derive entity name from schema filename
    let schema_name = schema_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .trim_end_matches(".schema");
    
    let entity_name = schema_name.trim_end_matches("Model");
    
    // Base directory for resolving $refs
    let schema_dir = schema_path.parent().unwrap();
    
    let mut imports = HashSet::new();
    imports.insert("sea_orm::entity::prelude::*".to_string());
    imports.insert("serde::{Serialize, Deserialize}".to_string());
    
    let type_mapper = TypeMapper::new();
    
    // Parse properties
    let properties = schema.get("properties")
        .and_then(|v| v.as_object())
        .map(|o| o.clone())
        .unwrap_or_default();
    
    let required: HashSet<String> = schema.get("required")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    
    // Collect external enums referenced by this schema
    let mut external_enums = Vec::new();
    for (_name, prop) in &properties {
        if let Some(ref_path) = prop.get("$ref").and_then(|v| v.as_str()) {
            if let Some(enum_info) = try_load_external_enum(schema_dir, ref_path) {
                // Only add if not already present
                if !external_enums.iter().any(|e: &EnumInfo| e.name == enum_info.name) {
                    external_enums.push(enum_info);
                }
            }
        }
    }
    
    // Parse fields - collect all first, then sort with id fields first
    let mut fields = Vec::new();
    let primary_keys: Vec<String> = match &extensions.primary_key {
        PrimaryKey::Single(k) => vec![k.clone()],
        PrimaryKey::Composite(keys) => keys.clone(),
    };
    
    for (name, prop) in &properties {
        let is_required = required.contains(name);
        let mut field = type_mapper.map_property(name, prop, is_required, &mut imports);
        
        // Check if this is a primary key
        if primary_keys.contains(name) {
            field.is_primary_key = true;
            field.sea_orm_attrs.insert(0, "primary_key".to_string());
            field.sea_orm_attrs.push("auto_increment = false".to_string());
        }
        
        // Check for unique constraint
        if prop.get("x-familiar-unique").and_then(|v| v.as_bool()) == Some(true) {
            field.sea_orm_attrs.push("unique".to_string());
        }
        
        // Check for default value
        if let Some(default_val) = prop.get("x-familiar-default-value") {
            if let Some(num) = default_val.as_i64() {
                field.sea_orm_attrs.push(format!("default_value = {}", num));
            }
        }
        
        fields.push(field);
    }
    
    // Sort fields: primary keys first, then by name
    fields.sort_by(|a, b| {
        match (a.is_primary_key, b.is_primary_key) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });
    
    // Collect enum names from external refs (prefer these - they have x-familiar-sea-orm config)
    let external_enum_names: std::collections::HashSet<String> = external_enums.iter()
        .map(|e| e.name.clone())
        .collect();
    
    // Parse enums from inline definitions, but skip any that duplicate external enums
    let inline_enums: Vec<EnumInfo> = parse_enums(&schema)
        .into_iter()
        .filter(|e| !external_enum_names.contains(&e.name))
        .collect();
    
    // Combine: external enums first (canonical), then any unique inline enums
    let mut enums = external_enums;
    enums.extend(inline_enums);
    
    Ok(ParsedEntity {
        schema_name: schema_name.to_string(),
        entity_name: entity_name.to_string(),
        table_name: extensions.table,
        module: extensions.module,
        fields,
        enums,
        relations: extensions.relations,
        imports,
    })
}

/// Try to load an external enum schema and extract SeaORM enum info
fn try_load_external_enum(schema_dir: &Path, ref_path: &str) -> Option<EnumInfo> {
    // Resolve the $ref path relative to the schema directory
    let full_path = schema_dir.join(ref_path);
    let content = fs::read_to_string(&full_path).ok()?;
    let schema: Value = serde_json::from_str(&content).ok()?;
    
    // Check for x-familiar-sea-orm extension
    let sea_orm = schema.get("x-familiar-sea-orm")?;
    let values = sea_orm.get("values")?.as_object()?;
    
    // Get enum name from title
    let name = schema.get("title")?.as_str()?;
    
    let variants: Vec<EnumVariant> = values.iter()
        .map(|(rust_name, db_val)| EnumVariant {
            rust_name: rust_name.clone(),
            db_value: db_val.as_str().unwrap_or(rust_name).to_string(),
        })
        .collect();
    
    Some(EnumInfo {
        name: name.to_string(),
        variants,
    })
}

fn parse_enums(schema: &Value) -> Vec<EnumInfo> {
    let mut enums = Vec::new();
    
    if let Some(defs) = schema.get("definitions").and_then(|v| v.as_object()) {
        for (name, def) in defs {
            if let Some(variants) = def.get("enum").and_then(|v| v.as_array()) {
                let enum_variants: Vec<EnumVariant> = variants
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| {
                        // Convert PascalCase to snake_case for DB value
                        let db_value = to_snake_case(s);
                        EnumVariant {
                            rust_name: s.to_string(),
                            db_value,
                        }
                    })
                    .collect();
                
                if !enum_variants.is_empty() {
                    enums.push(EnumInfo {
                        name: name.clone(),
                        variants: enum_variants,
                    });
                }
            }
        }
    }
    
    enums
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

/// Generate Rust code for an entity
pub fn generate_entity_code(entity: &ParsedEntity, entity_module_map: &HashMap<String, String>) -> String {
    let mut code = String::new();
    
    // Header
    code.push_str(&format!(
        "//! SeaORM entity: {}\n\
         //! Generated from: database/{}Model.schema.json\n\
         //! DO NOT EDIT - Regenerate with: cargo xtask codegen sea-entities\n\n",
        entity.entity_name, entity.entity_name
    ));
    
    // Imports
    code.push_str("use sea_orm::entity::prelude::*;\n");
    code.push_str("use serde::{Serialize, Deserialize};\n");
    
    // Collect primitive imports
    let primitive_imports: Vec<&String> = entity.imports.iter()
        .filter(|i| i.starts_with("familiar_primitives::"))
        .collect();
    
    if !primitive_imports.is_empty() {
        let types: Vec<&str> = primitive_imports.iter()
            .map(|i| i.split("::").last().unwrap_or(""))
            .collect();
        code.push_str(&format!("use familiar_primitives::{{{}}};\n", types.join(", ")));
    }
    
    code.push('\n');
    
    // Generate enums
    for enum_info in &entity.enums {
        code.push_str(&generate_enum_code(enum_info));
        code.push('\n');
    }
    
    // Model struct
    code.push_str("#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]\n");
    code.push_str(&format!("#[sea_orm(table_name = \"{}\")]\n", entity.table_name));
    code.push_str("pub struct Model {\n");
    
    for field in &entity.fields {
        // Build sea_orm attribute
        if !field.sea_orm_attrs.is_empty() {
            code.push_str(&format!("    #[sea_orm({})]\n", field.sea_orm_attrs.join(", ")));
        }
        code.push_str(&format!("    pub {}: {},\n", field.name, field.rust_type));
    }
    
    code.push_str("}\n\n");
    
    // Relations enum
    code.push_str("#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]\n");
    code.push_str("pub enum Relation {\n");
    
    for rel in &entity.relations {
        code.push_str(&generate_relation_attr(rel, entity, entity_module_map));
    }
    
    code.push_str("}\n\n");
    
    // Related impls
    for rel in &entity.relations {
        if let Some(impl_code) = generate_related_impl(rel, entity, entity_module_map) {
            code.push_str(&impl_code);
        }
    }
    
    // ActiveModelBehavior
    code.push_str("impl ActiveModelBehavior for ActiveModel {}\n");
    
    code
}

fn generate_enum_code(enum_info: &EnumInfo) -> String {
    let mut code = String::new();
    
    code.push_str("#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum, schemars::JsonSchema)]\n");
    code.push_str("#[sea_orm(rs_type = \"String\", db_type = \"String(StringLen::None)\")]\n");
    code.push_str(&format!("pub enum {} {{\n", enum_info.name));
    
    for variant in &enum_info.variants {
        code.push_str(&format!("    #[sea_orm(string_value = \"{}\")]\n", variant.db_value));
        code.push_str(&format!("    {},\n", variant.rust_name));
    }
    
    code.push_str("}\n");
    code
}

fn generate_relation_attr(rel: &Relation, entity: &ParsedEntity, entity_module_map: &HashMap<String, String>) -> String {
    let target_entity = rel.target.trim_end_matches("Model");
    // Use simplified module name matching hand-written convention
    let target_file_module = simplify_module_name(target_entity);
    
    // Determine if cross-module relation
    let target_module = entity_module_map.get(target_entity).cloned().unwrap_or_else(|| entity.module.clone());
    let is_cross_module = target_module != entity.module;
    
    // Generate the correct entity path based on whether it's cross-module
    let entity_path = if is_cross_module {
        format!("crate::entities::db::{}::{}::Entity", target_module, target_file_module)
    } else {
        format!("super::{}::Entity", target_file_module)
    };
    
    let column_path = if is_cross_module {
        format!("crate::entities::db::{}::{}::Column", target_module, target_file_module)
    } else {
        format!("super::{}::Column", target_file_module)
    };
    
    match rel.rel_type.as_str() {
        "has_many" => {
            let field_name = rel.field.as_ref()
                .map(|f| to_pascal_case(f))
                .unwrap_or_else(|| format!("{}s", target_entity));
            format!(
                "    #[sea_orm(has_many = \"{}\")]\n    {},\n",
                entity_path, field_name
            )
        }
        "has_one" => {
            let field_name = rel.field.as_ref()
                .map(|f| to_pascal_case(f))
                .unwrap_or_else(|| target_entity.to_string());
            format!(
                "    #[sea_orm(has_one = \"{}\")]\n    {},\n",
                entity_path, field_name
            )
        }
        "belongs_to" => {
            let from_col = rel.from.as_ref().map(|f| to_pascal_case(f)).unwrap_or_default();
            let to_col = rel.to.as_ref().map(|f| to_pascal_case(f)).unwrap_or_else(|| "Id".to_string());
            format!(
                "    #[sea_orm(\n        belongs_to = \"{}\",\n        from = \"Column::{}\",\n        to = \"{}::{}\"\n    )]\n    {},\n",
                entity_path, from_col, column_path, to_col, target_entity
            )
        }
        _ => String::new(),
    }
}

/// Simplify module name to match hand-written conventions
/// e.g., "AuthSession" -> "session", "ConsentRecord" -> "consent"
fn simplify_module_name(entity_name: &str) -> String {
    // Handle special cases first
    match entity_name {
        "AuthSession" => "session".to_string(),
        "ConsentRecord" => "consent".to_string(),
        "FamilyInvitation" => "invitation".to_string(),
        "AuditLogEntry" => "audit".to_string(),
        "EntityRegistry" => "entity_registry".to_string(),
        "ContentPayload" => "content".to_string(),
        _ => to_snake_case(entity_name),
    }
}

fn generate_related_impl(rel: &Relation, entity: &ParsedEntity, entity_module_map: &HashMap<String, String>) -> Option<String> {
    let target_entity = rel.target.trim_end_matches("Model");
    let target_file_module = simplify_module_name(target_entity);
    
    // Determine if cross-module relation
    let target_module = entity_module_map.get(target_entity).cloned().unwrap_or_else(|| entity.module.clone());
    let is_cross_module = target_module != entity.module;
    
    // Generate the correct entity path based on whether it's cross-module
    let entity_path = if is_cross_module {
        format!("crate::entities::db::{}::{}::Entity", target_module, target_file_module)
    } else {
        format!("super::{}::Entity", target_file_module)
    };
    
    let field_name = match rel.rel_type.as_str() {
        "has_many" => rel.field.as_ref()
            .map(|f| to_pascal_case(f))
            .unwrap_or_else(|| format!("{}s", target_entity)),
        "has_one" => rel.field.as_ref()
            .map(|f| to_pascal_case(f))
            .unwrap_or_else(|| target_entity.to_string()),
        "belongs_to" => target_entity.to_string(),
        _ => return None,
    };
    
    Some(format!(
        "impl Related<{}> for Entity {{\n    fn to() -> RelationDef {{\n        Relation::{}.def()\n    }}\n}}\n\n",
        entity_path, field_name
    ))
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// Generate all SeaORM entities from database schemas
pub fn generate_all(schema_dir: &Path, output_dir: &Path, validate_only: bool) -> anyhow::Result<GenerationResult> {
    let database_dir = schema_dir.join("database");
    
    if !database_dir.exists() {
        anyhow::bail!("Database schema directory not found: {}", database_dir.display());
    }
    
    let mut result = GenerationResult::default();
    let mut entities_by_module: HashMap<String, Vec<ParsedEntity>> = HashMap::new();
    // Map entity name (e.g., "Tenant") -> module (e.g., "conversation")
    let mut entity_module_map: HashMap<String, String> = HashMap::new();
    
    // Parse all database schemas
    for entry in fs::read_dir(&database_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            match parse_schema(&path) {
                Ok(entity) => {
                    // Build module lookup map
                    entity_module_map.insert(entity.entity_name.clone(), entity.module.clone());
                    entities_by_module
                        .entry(entity.module.clone())
                        .or_default()
                        .push(entity);
                    result.parsed += 1;
                }
                Err(e) => {
                    result.errors.push(format!("Failed to parse {}: {}", path.display(), e));
                }
            }
        }
    }
    
    if validate_only {
        // Just return parse results without writing
        return Ok(result);
    }
    
    // Generate files by module
    for (module, entities) in &entities_by_module {
        let module_dir = output_dir.join(module);
        fs::create_dir_all(&module_dir)?;
        
        for entity in entities {
            let code = generate_entity_code(entity, &entity_module_map);
            // Use simplified file name matching hand-written conventions
            let file_name = format!("{}.rs", simplify_module_name(&entity.entity_name));
            let file_path = module_dir.join(&file_name);
            
            fs::write(&file_path, &code)?;
            result.generated.push(file_path.display().to_string());
        }
        
        // Generate module mod.rs
        let mod_content = generate_module_mod(entities);
        fs::write(module_dir.join("mod.rs"), mod_content)?;
    }
    
    // Generate root mod.rs
    let root_mod = generate_root_mod(&entities_by_module);
    fs::write(output_dir.join("mod.rs"), root_mod)?;
    
    Ok(result)
}

fn generate_module_mod(entities: &[ParsedEntity]) -> String {
    let mut code = String::from("//! Generated SeaORM entities\n//! DO NOT EDIT\n\n");
    
    for entity in entities {
        let mod_name = simplify_module_name(&entity.entity_name);
        code.push_str(&format!("pub mod {};\n", mod_name));
    }
    
    code
}

fn generate_root_mod(modules: &HashMap<String, Vec<ParsedEntity>>) -> String {
    let mut code = String::from(
        "//! SeaORM Database Entities\n\
         //! Generated from database schemas - DO NOT EDIT\n\
         //! Regenerate with: cargo xtask codegen sea-entities\n\n"
    );
    
    for module in modules.keys() {
        code.push_str(&format!("pub mod {};\n", module));
    }
    
    code.push_str("\n// Re-export common entities for convenience\n");
    
    for (module, entities) in modules {
        let exports: Vec<String> = entities.iter()
            .map(|e| format!("{}::{}", module, simplify_module_name(&e.entity_name)))
            .collect();
        code.push_str(&format!("pub use {{{}}};\n", exports.join(", ")));
    }
    
    code
}

#[derive(Debug, Default)]
pub struct GenerationResult {
    pub parsed: usize,
    pub generated: Vec<String>,
    pub errors: Vec<String>,
}

/// Validate generated entities against existing hand-written ones
pub fn validate_parity(
    generated_dir: &Path,
    handwritten_dir: &Path,
) -> anyhow::Result<ParityResult> {
    let mut result = ParityResult::default();
    
    // Walk through generated modules
    for module in &["auth", "conversation", "physics"] {
        let gen_module = generated_dir.join(module);
        let hw_module = handwritten_dir.join(module);
        
        if !gen_module.exists() || !hw_module.exists() {
            continue;
        }
        
        for entry in fs::read_dir(&gen_module)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if file_name == "mod.rs" {
                continue;
            }
            
            let hw_path = hw_module.join(file_name);
            
            if hw_path.exists() {
                let comparison = compare_entity_files(&path, &hw_path)?;
                if comparison.is_match() {
                    result.matches.push(format!("{}/{}", module, file_name));
                } else {
                    result.mismatches.push(EntityMismatch {
                        file: format!("{}/{}", module, file_name),
                        differences: comparison.differences,
                    });
                }
            } else {
                result.new_entities.push(format!("{}/{}", module, file_name));
            }
        }
    }
    
    Ok(result)
}

#[derive(Debug, Default)]
pub struct ParityResult {
    pub matches: Vec<String>,
    pub mismatches: Vec<EntityMismatch>,
    pub new_entities: Vec<String>,
}

#[derive(Debug)]
pub struct EntityMismatch {
    pub file: String,
    pub differences: Vec<String>,
}

struct EntityComparison {
    differences: Vec<String>,
}

impl EntityComparison {
    fn is_match(&self) -> bool {
        self.differences.is_empty()
    }
}

fn compare_entity_files(generated: &Path, handwritten: &Path) -> anyhow::Result<EntityComparison> {
    let gen_content = fs::read_to_string(generated)?;
    let hw_content = fs::read_to_string(handwritten)?;
    
    let mut differences = Vec::new();
    
    // Extract and compare table names
    let gen_table = extract_table_name(&gen_content);
    let hw_table = extract_table_name(&hw_content);
    if gen_table != hw_table {
        differences.push(format!("Table name: generated='{}' vs handwritten='{}'", 
            gen_table.unwrap_or("none"), hw_table.unwrap_or("none")));
    }
    
    // Extract and compare field counts
    let gen_fields = count_model_fields(&gen_content);
    let hw_fields = count_model_fields(&hw_content);
    if gen_fields != hw_fields {
        differences.push(format!("Field count: generated={} vs handwritten={}", gen_fields, hw_fields));
    }
    
    // Extract and compare relation counts
    let gen_relations = count_relations(&gen_content);
    let hw_relations = count_relations(&hw_content);
    if gen_relations != hw_relations {
        differences.push(format!("Relation count: generated={} vs handwritten={}", gen_relations, hw_relations));
    }
    
    Ok(EntityComparison { differences })
}

fn extract_table_name(content: &str) -> Option<&str> {
    content.lines()
        .find(|line| line.contains("table_name"))
        .and_then(|line| {
            line.split('"').nth(1)
        })
}

fn count_model_fields(content: &str) -> usize {
    let mut in_model = false;
    let mut count = 0;
    
    for line in content.lines() {
        if line.contains("pub struct Model") {
            in_model = true;
            continue;
        }
        if in_model {
            if line.trim() == "}" {
                break;
            }
            if line.trim().starts_with("pub ") && line.contains(":") {
                count += 1;
            }
        }
    }
    count
}

fn count_relations(content: &str) -> usize {
    content.lines()
        .filter(|line| line.contains("#[sea_orm(has_many") || 
                       line.contains("#[sea_orm(has_one") ||
                       line.contains("belongs_to ="))
        .count()
}


//! MCP Tool Implementations
//!
//! Implements the MCP server handler with tools for schema queries.
//! Uses mcp-attr for declarative tool definitions.

use crate::mcp::graph::SchemaGraph;
use mcp_attr::server::{mcp_server, McpServer};
use mcp_attr::Result;
use petgraph::Direction;
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;

/// MCP Server for schema tools
pub struct SchemaTools {
    graph: Arc<SchemaGraph>,
}

impl SchemaTools {
    pub fn new(graph: SchemaGraph) -> Self {
        Self {
            graph: Arc::new(graph),
        }
    }
}

// Response types for better serialization
#[derive(Debug, Serialize)]
struct StatusResponse {
    bundle_hash: String,
    schema_count: usize,
    edge_count: usize,
    scc_count: usize,
    artifact_count: usize,
}

#[derive(Debug, Serialize)]
struct ResolveResponse {
    resolved: bool,
    id: Option<String>,
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct TypeResponse {
    id: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    service: Option<String>,
    fields: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    codegen: Option<serde_json::Value>,
    refs_out_count: usize,
    refs_in_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    closure: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct ClosureResponse {
    id: String,
    direction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    depth: Option<usize>,
    nodes: Vec<serde_json::Value>,
    scc_groups: Vec<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct ImportsResponse {
    id: String,
    lang: String,
    imports: Vec<String>,
}

#[mcp_server]
impl McpServer for SchemaTools {
    // ========== Meta Tools ==========
    
    /// Get registry status: bundle_hash, schema_count, edge_count, scc_count, artifact_count
    #[tool]
    async fn status(&self) -> Result<String> {
        let response = StatusResponse {
            bundle_hash: self.graph.bundle_hash.clone(),
            schema_count: self.graph.schema_count(),
            edge_count: self.graph.edge_count(),
            scc_count: self.graph.scc_count(),
            artifact_count: self.graph.artifact_count(),
        };
        Ok(serde_json::to_string_pretty(&response)?)
    }
    
    /// Resolve a query (name/path/id) to canonical $id
    #[tool]
    async fn resolve(&self, query: String) -> Result<String> {
        let response = match self.graph.resolve(&query) {
            Some(id) => ResolveResponse {
                resolved: true,
                id: Some(id.clone()),
                query,
                error: None,
            },
            None => ResolveResponse {
                resolved: false,
                id: None,
                query: query.clone(),
                error: Some(format!("No schema found matching query: {}", query)),
            },
        };
        Ok(serde_json::to_string_pretty(&response)?)
    }
    
    /// Get the full raw JSON schema (use sparingly, prefer get_type)
    #[tool]
    async fn schema_raw(&self, id: String) -> Result<String> {
        let resolved_id = self.graph.resolve(&id)
            .cloned()
            .unwrap_or_else(|| id.clone());
        
        match self.graph.get_raw(&resolved_id) {
            Some(raw) => Ok(serde_json::to_string_pretty(&json!({
                "id": resolved_id,
                "schema": raw
            }))?),
            None => Ok(serde_json::to_string_pretty(&json!({
                "error": format!("Schema not found: {}", id)
            }))?),
        }
    }
    
    // ========== Query Tools ==========
    
    /// Get schema metadata + immediate fields (shallow response by default)
    #[tool]
    async fn get_type(
        &self, 
        id: String,
        #[doc = "Include full JSON schema"] 
        include_schema: Option<bool>,
        #[doc = "Include transitive dependencies"]
        include_closure: Option<bool>,
    ) -> Result<String> {
        let resolved_id = self.graph.resolve(&id)
            .cloned()
            .unwrap_or_else(|| id.clone());
        
        let Some(node) = self.graph.get(&resolved_id) else {
            return Ok(serde_json::to_string_pretty(&json!({
                "error": format!("Schema not found: {}", id)
            }))?);
        };
        
        let refs_out = self.graph.refs_out(&resolved_id);
        let refs_in = self.graph.refs_in(&resolved_id);
        
        let mut response = TypeResponse {
            id: node.id.clone(),
            path: node.path.to_string_lossy().to_string(),
            title: node.title.clone(),
            kind: node.kind.clone(),
            service: node.service.clone(),
            fields: node.fields.iter().map(|f| serde_json::to_value(f).unwrap_or_default()).collect(),
            codegen: node.codegen.as_ref().map(|c| serde_json::to_value(c).unwrap_or_default()),
            refs_out_count: refs_out.len(),
            refs_in_count: refs_in.len(),
            schema: None,
            closure: None,
        };
        
        if include_schema.unwrap_or(false) {
            response.schema = self.graph.get_raw(&resolved_id).cloned();
        }
        
        if include_closure.unwrap_or(false) {
            let closure = self.graph.closure(&resolved_id, Direction::Outgoing, None);
            response.closure = Some(serde_json::to_value(closure)?);
        }
        
        Ok(serde_json::to_string_pretty(&response)?)
    }
    
    /// Get immediate outgoing $refs (what this schema depends on)
    #[tool]
    async fn get_refs(&self, id: String) -> Result<String> {
        let resolved_id = self.graph.resolve(&id)
            .cloned()
            .unwrap_or_else(|| id.clone());
        
        let refs = self.graph.refs_out(&resolved_id);
        Ok(serde_json::to_string_pretty(&json!({
            "id": resolved_id,
            "refs": refs
        }))?)
    }
    
    /// Get immediate incoming refs (what depends on this schema)
    #[tool]
    async fn get_dependents(&self, id: String) -> Result<String> {
        let resolved_id = self.graph.resolve(&id)
            .cloned()
            .unwrap_or_else(|| id.clone());
        
        let dependents = self.graph.refs_in(&resolved_id);
        Ok(serde_json::to_string_pretty(&json!({
            "id": resolved_id,
            "dependents": dependents
        }))?)
    }
    
    /// Fuzzy search schemas by name/path
    #[tool]
    async fn search(&self, query: String, limit: Option<usize>) -> Result<String> {
        let results = self.graph.search(&query, limit.unwrap_or(10));
        Ok(serde_json::to_string_pretty(&json!({
            "query": query,
            "count": results.len(),
            "results": results
        }))?)
    }
    
    /// List all schemas by x-familiar-kind, or list all kinds if no kind specified
    #[tool]
    async fn list_kinds(&self, kind: Option<String>) -> Result<String> {
        match kind {
            Some(k) => {
                let schemas = self.graph.list_by_kind(&k);
                Ok(serde_json::to_string_pretty(&json!({
                    "kind": k,
                    "count": schemas.len(),
                    "schemas": schemas
                }))?)
            }
            None => {
                let kinds = self.graph.all_kinds();
                Ok(serde_json::to_string_pretty(&json!({
                    "kinds": kinds
                }))?)
            }
        }
    }
    
    // ========== Agent Workflow Tools ==========
    
    /// Get transitive dependencies or dependents with depth + SCC boundaries
    #[tool]
    async fn closure(
        &self, 
        id: String,
        #[doc = "out = dependencies, in = dependents"]
        direction: Option<String>,
        #[doc = "Max depth (default: unlimited)"]
        depth: Option<usize>,
    ) -> Result<String> {
        let resolved_id = self.graph.resolve(&id)
            .cloned()
            .unwrap_or_else(|| id.clone());
        
        let dir_str = direction.as_deref().unwrap_or("out");
        let dir = match dir_str {
            "in" => Direction::Incoming,
            _ => Direction::Outgoing,
        };
        
        let nodes = self.graph.closure(&resolved_id, dir, depth);
        let scc_groups: Vec<Vec<String>> = self.graph.scc_groups()
            .iter()
            .map(|g| g.clone())
            .collect();
        
        let response = ClosureResponse {
            id: resolved_id,
            direction: dir_str.to_string(),
            depth,
            nodes: nodes.iter().map(|n| serde_json::to_value(n).unwrap_or_default()).collect(),
            scc_groups,
        };
        
        Ok(serde_json::to_string_pretty(&response)?)
    }
    
    /// Generate import statements for Rust/TypeScript/Python
    #[tool]
    async fn imports_for(
        &self, 
        id: String,
        #[doc = "Target language: rust, typescript, ts, python, py"]
        lang: String,
    ) -> Result<String> {
        let resolved_id = self.graph.resolve(&id)
            .cloned()
            .unwrap_or_else(|| id.clone());
        
        let imports = self.graph.imports_for(&resolved_id, &lang);
        
        let response = ImportsResponse {
            id: resolved_id,
            lang,
            imports,
        };
        
        Ok(serde_json::to_string_pretty(&response)?)
    }
    
    /// Check for union/enum ambiguity issues
    #[tool]
    async fn lint_unions(&self, id: String) -> Result<String> {
        let resolved_id = self.graph.resolve(&id)
            .cloned()
            .unwrap_or_else(|| id.clone());
        
        let warnings = self.graph.lint_unions(&resolved_id);
        Ok(serde_json::to_string_pretty(&json!({
            "id": resolved_id,
            "warnings": warnings,
            "warning_count": warnings.len()
        }))?)
    }
    
    /// Find which services/operations reference this schema
    #[tool]
    async fn services_for_schema(&self, id: String) -> Result<String> {
        let resolved_id = self.graph.resolve(&id)
            .cloned()
            .unwrap_or_else(|| id.clone());
        
        // Find all schemas that reference this one and have a service
        let dependents = self.graph.refs_in(&resolved_id);
        let mut services: Vec<serde_json::Value> = Vec::new();
        let mut unique_services: std::collections::HashSet<String> = std::collections::HashSet::new();
        
        for dep_id in dependents {
            if let Some(node) = self.graph.get(dep_id) {
                if let Some(service) = &node.service {
                    unique_services.insert(service.clone());
                    services.push(json!({
                        "service": service,
                        "schema_id": dep_id,
                        "kind": node.kind
                    }));
                }
            }
        }
        
        // Also check if this schema itself has a service
        if let Some(node) = self.graph.get(&resolved_id) {
            if let Some(service) = &node.service {
                unique_services.insert(service.clone());
                services.insert(0, json!({
                    "service": service,
                    "schema_id": resolved_id.clone(),
                    "kind": node.kind
                }));
            }
        }
        
        let unique_list: Vec<String> = unique_services.into_iter().collect();
        
        Ok(serde_json::to_string_pretty(&json!({
            "id": resolved_id,
            "services": unique_list,
            "references": services
        }))?)
    }
    
    // ========== Graph Analysis Tools (v3.1) ==========
    
    /// Get SCC (strongly connected component) report with cycle info and boxing suggestions
    #[tool]
    async fn scc_report(&self) -> Result<String> {
        let scc_groups = self.graph.scc_groups();
        let mut cycles: Vec<serde_json::Value> = Vec::new();
        
        for (idx, group) in scc_groups.iter().enumerate() {
            if group.len() > 1 {
                // Multi-member SCC = cycle
                cycles.push(json!({
                    "scc_id": idx,
                    "members": group,
                    "size": group.len(),
                    "suggested_box_edge": format!("{} -> {}", group[0], group[1]),
                    "has_unions": group.iter().any(|id| {
                        self.graph.get(id).map(|n| 
                            n.codegen.as_ref().map(|c| 
                                c.enum_repr.is_some()
                            ).unwrap_or(false)
                        ).unwrap_or(false)
                    })
                }));
            } else if group.len() == 1 {
                // Check for self-loop
                let id = &group[0];
                let refs_out = self.graph.refs_out(id);
                if refs_out.iter().any(|r| *r == id) {
                    cycles.push(json!({
                        "scc_id": idx,
                        "members": group,
                        "size": 1,
                        "self_referential": true,
                        "suggested_box_edge": format!("{} -> {}", id, id),
                    }));
                }
            }
        }
        
        Ok(serde_json::to_string_pretty(&json!({
            "total_sccs": scc_groups.len(),
            "cycles": cycles,
            "cycle_count": cycles.len()
        }))?)
    }
    
    /// Get hub report (top N high-degree types to watch)
    #[tool]
    async fn hub_report(
        &self,
        #[doc = "Number of top hubs to return (default: 10)"]
        top_n: Option<usize>,
    ) -> Result<String> {
        let n = top_n.unwrap_or(10);
        let mut nodes: Vec<(String, usize, usize)> = Vec::new();
        
        for schema_id in self.graph.all_ids() {
            let in_degree = self.graph.refs_in(schema_id).len();
            let out_degree = self.graph.refs_out(schema_id).len();
            nodes.push((schema_id.to_string(), in_degree, out_degree));
        }
        
        // Sort by in_degree descending (most depended upon)
        nodes.sort_by(|a, b| b.1.cmp(&a.1));
        nodes.truncate(n);
        
        let hubs: Vec<serde_json::Value> = nodes.iter()
            .map(|(id, in_deg, out_deg)| {
                let kind = self.graph.get(id).and_then(|n| n.kind.clone());
                json!({
                    "id": id,
                    "in_degree": in_deg,
                    "out_degree": out_deg,
                    "total_degree": in_deg + out_deg,
                    "kind": kind,
                    "suggested_derives": if *in_deg >= 3 { vec!["Eq", "Hash"] } else { vec![] }
                })
            })
            .collect();
        
        Ok(serde_json::to_string_pretty(&json!({
            "top_n": n,
            "hubs": hubs
        }))?)
    }
    
    /// Get hoist candidates (duplicate inline schemas - PROPOSAL ONLY, never auto-applied)
    #[tool]
    async fn hoist_candidates(&self) -> Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut shape_map: std::collections::HashMap<u64, Vec<(String, String)>> = std::collections::HashMap::new();
        
        // Scan all schemas for inline object definitions
        for schema_id in self.graph.all_ids() {
            if let Some(raw) = self.graph.get_raw(schema_id) {
                if let Some(props) = raw.get("properties").and_then(|p| p.as_object()) {
                    for (prop_name, prop_schema) in props {
                        // Check if it's an inline object (not a $ref)
                        if prop_schema.get("$ref").is_none() 
                            && prop_schema.get("properties").is_some() 
                        {
                            // Hash the normalized shape
                            let normalized = normalize_for_hash(prop_schema);
                            let mut hasher = DefaultHasher::new();
                            normalized.hash(&mut hasher);
                            let hash = hasher.finish();
                            
                            shape_map.entry(hash)
                                .or_default()
                                .push((schema_id.to_string(), prop_name.clone()));
                        }
                    }
                }
            }
        }
        
        // Only report duplicates
        let candidates: Vec<serde_json::Value> = shape_map
            .into_iter()
            .filter(|(_, v)| v.len() > 1)
            .map(|(hash, occurrences)| {
                let suggested_name = format!(
                    "{}{}",
                    occurrences[0].0.split('/').last().unwrap_or(&occurrences[0].0),
                    pascal_case(&occurrences[0].1)
                );
                json!({
                    "shape_hash": format!("{:016x}", hash),
                    "occurrence_count": occurrences.len(),
                    "occurrences": occurrences.iter().map(|(s, p)| format!("{}.{}", s, p)).collect::<Vec<_>>(),
                    "suggested_name": suggested_name,
                    "action": "PROPOSAL ONLY - manual review required"
                })
            })
            .collect();
        
        Ok(serde_json::to_string_pretty(&json!({
            "candidate_count": candidates.len(),
            "candidates": candidates,
            "note": "Hoist candidates are proposals only. They do NOT affect codegen until schemas are manually updated."
        }))?)
    }
    
    // ========== Debugging/Explanation Tools (v3.1) ==========
    
    /// Explain why a type is boxed (SCC membership, recursion)
    #[tool]
    async fn explain_boxing(&self, id: String) -> Result<String> {
        let resolved_id = self.graph.resolve(&id)
            .cloned()
            .unwrap_or_else(|| id.clone());
        
        let scc_groups = self.graph.scc_groups();
        
        // Find which SCC this type belongs to
        let mut in_scc: Option<&Vec<String>> = None;
        let mut scc_idx = 0;
        for (idx, group) in scc_groups.iter().enumerate() {
            if group.contains(&resolved_id) && group.len() > 1 {
                in_scc = Some(group);
                scc_idx = idx;
                break;
            }
        }
        
        // Check for self-reference
        let refs_out = self.graph.refs_out(&resolved_id);
        let self_ref = refs_out.iter().any(|r| *r == &resolved_id);
        
        // Get recursion override if any
        let recursion_override = self.graph.get_raw(&resolved_id)
            .and_then(|s| s.get("x-familiar-rust-recursion").cloned());
        
        let mut explanation = Vec::new();
        let mut should_box = false;
        let mut box_reason = String::new();
        
        if let Some(group) = in_scc {
            should_box = true;
            box_reason = format!("Part of SCC #{} with {} members", scc_idx, group.len());
            explanation.push(json!({
                "reason": "scc_membership",
                "detail": format!("This type is part of a cycle with: {}", 
                    group.iter().filter(|g| *g != &resolved_id).cloned().collect::<Vec<_>>().join(", ")),
                "scc_id": scc_idx,
                "members": group
            }));
        } else if self_ref {
            should_box = true;
            box_reason = "Self-referential type".to_string();
            explanation.push(json!({
                "reason": "self_reference",
                "detail": "This type references itself directly"
            }));
        }
        
        if let Some(override_val) = &recursion_override {
            explanation.push(json!({
                "reason": "explicit_override",
                "detail": "Schema has x-familiar-rust-recursion override",
                "value": override_val
            }));
        }
        
        if !should_box {
            explanation.push(json!({
                "reason": "no_boxing_needed",
                "detail": "This type is not recursive and not part of any cycle"
            }));
        }
        
        Ok(serde_json::to_string_pretty(&json!({
            "id": resolved_id,
            "should_box": should_box,
            "box_reason": box_reason,
            "strategy": recursion_override.as_ref()
                .and_then(|v| v.get("strategy"))
                .and_then(|s| s.as_str())
                .unwrap_or(if should_box { "box" } else { "none" }),
            "explanation": explanation
        }))?)
    }
    
    /// Explain why specific derives are applied to a type
    #[tool]
    async fn explain_derives(&self, id: String) -> Result<String> {
        let resolved_id = self.graph.resolve(&id)
            .cloned()
            .unwrap_or_else(|| id.clone());
        
        let raw = self.graph.get_raw(&resolved_id);
        
        // Default derives
        let defaults = vec!["Debug", "Clone", "Serialize", "Deserialize", "JsonSchema"];
        
        // Get policy
        let policy = raw.as_ref()
            .and_then(|s| s.get("x-familiar-rust-derive-policy"))
            .and_then(|p| p.as_str())
            .unwrap_or("strict");
        
        // Get explicit overrides
        let full_override = raw.as_ref()
            .and_then(|s| s.get("x-familiar-rust-derives"))
            .and_then(|d| d.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>());
        
        let additions = raw.as_ref()
            .and_then(|s| s.get("x-familiar-rust-derive-add"))
            .and_then(|d| d.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
            .unwrap_or_default();
        
        let exclusions = raw.as_ref()
            .and_then(|s| s.get("x-familiar-rust-derive-exclude"))
            .and_then(|d| d.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
            .unwrap_or_default();
        
        // Calculate graph suggestions
        let in_degree = self.graph.refs_in(&resolved_id).len();
        let mut graph_suggests: Vec<String> = Vec::new();
        let mut graph_reason = String::new();
        
        if in_degree >= 3 {
            // Check if Eq/Hash are safe
            let is_safe = raw.as_ref().map(|s| is_eq_hash_safe(s)).unwrap_or(false);
            if is_safe {
                graph_suggests.push("Eq".to_string());
                graph_suggests.push("Hash".to_string());
                graph_reason = format!("High in-degree ({}) and all fields support Eq/Hash", in_degree);
            } else {
                graph_reason = format!("High in-degree ({}) but fields contain f64/HashMap/Value - Eq/Hash not safe", in_degree);
            }
        }
        
        // Calculate final derives
        let final_derives = if let Some(full) = &full_override {
            full.clone()
        } else {
            let mut result: Vec<String> = defaults.iter().map(|s| s.to_string()).collect();
            
            // Add explicit additions
            for add in &additions {
                if !result.contains(add) {
                    result.push(add.clone());
                }
            }
            
            // Remove exclusions
            for exc in &exclusions {
                result.retain(|d| d != exc);
            }
            
            // Add graph suggestions only if policy allows
            if policy == "allow_graph_suggestions" {
                for suggest in &graph_suggests {
                    if !result.contains(suggest) {
                        result.push(suggest.clone());
                    }
                }
            }
            
            result
        };
        
        Ok(serde_json::to_string_pretty(&json!({
            "id": resolved_id,
            "policy": policy,
            "final_derives": final_derives,
            "explanation": {
                "defaults": defaults,
                "full_override": full_override,
                "additions": additions,
                "exclusions": exclusions,
                "graph_suggests": graph_suggests,
                "graph_reason": graph_reason,
                "graph_applied": policy == "allow_graph_suggestions" && !graph_suggests.is_empty()
            }
        }))?)
    }
    
    /// Get all policy settings that affect a type's codegen
    #[tool]
    async fn policy_for(&self, id: String) -> Result<String> {
        let resolved_id = self.graph.resolve(&id)
            .cloned()
            .unwrap_or_else(|| id.clone());
        
        let raw = self.graph.get_raw(&resolved_id);
        let node = self.graph.get(&resolved_id);
        
        // Extract all x-familiar-rust-* settings
        let mut policies: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        
        if let Some(schema) = raw {
            if let Some(obj) = schema.as_object() {
                for (k, v) in obj {
                    if k.starts_with("x-familiar-rust-") {
                        policies.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        
        // Compute derived properties
        let in_scc = self.graph.scc_groups().iter()
            .any(|g| g.contains(&resolved_id) && g.len() > 1);
        
        let in_degree = self.graph.refs_in(&resolved_id).len();
        let out_degree = self.graph.refs_out(&resolved_id).len();
        
        Ok(serde_json::to_string_pretty(&json!({
            "id": resolved_id,
            "explicit_policies": policies,
            "derived_properties": {
                "in_cycle": in_scc,
                "in_degree": in_degree,
                "out_degree": out_degree,
                "is_hub": in_degree >= 3,
                "kind": node.and_then(|n| n.kind.clone()),
                "service": node.and_then(|n| n.service.clone())
            },
            "effective_settings": {
                "derive_policy": policies.get("x-familiar-rust-derive-policy")
                    .and_then(|v| v.as_str())
                    .unwrap_or("strict"),
                "default_impl": policies.get("x-familiar-rust-default")
                    .and_then(|v| v.as_str())
                    .unwrap_or("derived"),
                "recursion_strategy": policies.get("x-familiar-rust-recursion")
                    .and_then(|v| v.get("strategy"))
                    .and_then(|s| s.as_str())
                    .unwrap_or(if in_scc { "box" } else { "none" }),
                "impl_modules": policies.get("x-familiar-rust-impl-ids")
            }
        }))?)
    }
    
    /// Analyze breaking changes between schema versions (requires two version paths)
    #[tool]
    async fn impact(
        &self, 
        id: String,
        #[doc = "Compare against this older version (e.g., 'v0.9.0')"]
        base_version: Option<String>,
    ) -> Result<String> {
        let resolved_id = self.graph.resolve(&id)
            .cloned()
            .unwrap_or_else(|| id.clone());
        
        // Get current schema (for future version comparison)
        let _current = self.graph.get_raw(&resolved_id);
        
        // For now, show what would be affected by changes to this schema
        let dependents = self.graph.refs_in(&resolved_id);
        let closure = self.graph.closure(&resolved_id, Direction::Incoming, None);
        
        // Categorize dependents by kind
        let mut by_kind: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        for dep_id in &dependents {
            if let Some(node) = self.graph.get(dep_id) {
                let kind = node.kind.clone().unwrap_or_else(|| "unknown".to_string());
                by_kind.entry(kind).or_default().push((*dep_id).clone());
            }
        }
        
        Ok(serde_json::to_string_pretty(&json!({
            "id": resolved_id,
            "base_version": base_version.unwrap_or_else(|| "current".to_string()),
            "note": "Full version diff requires loading both schema versions. This shows blast radius in current version.",
            "blast_radius": {
                "direct_dependents": dependents.len(),
                "transitive_dependents": closure.len(),
                "by_kind": by_kind
            },
            "dependents": dependents.iter().take(20).collect::<Vec<_>>(),
            "would_affect": format!("{} schemas total (showing first 20)", closure.len())
        }))?)
    }
    
    // ========== Schema Decomposition Tools ==========
    
    /// Find all schemas with embedded #/definitions/ and categorize them
    /// 
    /// Returns:
    /// - embedded_only: Definitions that have no standalone schema (need extraction)
    /// - duplicates: Definitions that duplicate existing standalone schemas (need $ref update)
    /// - self_referential: Definitions that reference themselves (keep as-is for recursion)
    #[tool]
    async fn find_embedded_definitions(&self) -> Result<String> {
        let mut embedded_only: Vec<serde_json::Value> = Vec::new();
        let mut duplicates: Vec<serde_json::Value> = Vec::new();
        let mut self_referential: Vec<serde_json::Value> = Vec::new();
        
        for schema_id in self.graph.all_ids() {
            if let Some(raw) = self.graph.get_raw(schema_id) {
                if let Some(defs) = raw.get("definitions").and_then(|d| d.as_object()) {
                    for (def_name, _def_schema) in defs {
                        // Check if standalone schema exists with exact title match
                        let search_results = self.graph.search(def_name, 5);
                        let standalone_exists = search_results
                            .iter()
                            .any(|r| r.title.as_ref().map(|t| t == def_name).unwrap_or(false));
                        
                        // Check if self-referential (definition name matches parent schema title)
                        let parent_title = raw.get("title")
                            .and_then(|t| t.as_str())
                            .unwrap_or("");
                        let is_self_ref = def_name == parent_title;
                        
                        if is_self_ref {
                            self_referential.push(json!({
                                "parent_schema": schema_id,
                                "definition_name": def_name,
                                "reason": "Recursive type - keep local for Box<T> support"
                            }));
                        } else if standalone_exists {
                            // Find the standalone schema path
                            let standalone_path = search_results
                                .iter()
                                .find(|r| r.title.as_ref().map(|t| t == def_name).unwrap_or(false))
                                .map(|r| r.id.clone())
                                .unwrap_or_else(|| "unknown".to_string());
                            
                            duplicates.push(json!({
                                "parent_schema": schema_id,
                                "definition_name": def_name,
                                "standalone_schema": standalone_path,
                                "action": "Update $ref to use standalone schema"
                            }));
                        } else {
                            embedded_only.push(json!({
                                "parent_schema": schema_id,
                                "definition_name": def_name,
                                "action": "Extract to standalone schema"
                            }));
                        }
                    }
                }
            }
        }
        
        // Sort for consistent output
        embedded_only.sort_by(|a, b| {
            let a_key = format!("{}/{}", 
                a.get("parent_schema").and_then(|v| v.as_str()).unwrap_or(""),
                a.get("definition_name").and_then(|v| v.as_str()).unwrap_or(""));
            let b_key = format!("{}/{}", 
                b.get("parent_schema").and_then(|v| v.as_str()).unwrap_or(""),
                b.get("definition_name").and_then(|v| v.as_str()).unwrap_or(""));
            a_key.cmp(&b_key)
        });
        
        duplicates.sort_by(|a, b| {
            let a_key = format!("{}/{}", 
                a.get("parent_schema").and_then(|v| v.as_str()).unwrap_or(""),
                a.get("definition_name").and_then(|v| v.as_str()).unwrap_or(""));
            let b_key = format!("{}/{}", 
                b.get("parent_schema").and_then(|v| v.as_str()).unwrap_or(""),
                b.get("definition_name").and_then(|v| v.as_str()).unwrap_or(""));
            a_key.cmp(&b_key)
        });
        
        Ok(serde_json::to_string_pretty(&json!({
            "summary": {
                "total_definitions": embedded_only.len() + duplicates.len() + self_referential.len(),
                "needs_extraction": embedded_only.len(),
                "duplicates_to_fix": duplicates.len(),
                "self_referential": self_referential.len()
            },
            "embedded_only": embedded_only,
            "duplicates": duplicates,
            "self_referential": self_referential
        }))?)
    }
    
    // ========== Codegen Artifact Tools ==========
    
    /// Get generated artifacts for a schema (Rust/TypeScript/Python file locations)
    #[tool]
    async fn get_artifacts(
        &self, 
        id: String,
        #[doc = "Filter by language: rust, typescript, python"]
        lang: Option<String>,
    ) -> Result<String> {
        let resolved_id = self.graph.resolve(&id)
            .cloned()
            .unwrap_or_else(|| id.clone());
        
        let artifacts = if let Some(l) = lang {
            self.graph.get_artifact(&resolved_id, &l)
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            self.graph.get_artifacts(&resolved_id)
        };
        
        Ok(serde_json::to_string_pretty(&json!({
            "id": resolved_id,
            "artifacts": artifacts.iter().map(|a| json!({
                "lang": a.lang,
                "file": a.file,
                "line": a.line,
                "type_name": a.type_name,
                "type_kind": a.type_kind
            })).collect::<Vec<_>>()
        }))?)
    }
    
    // NOTE: artifact_coverage() was removed - use graph_stats() instead which provides
    // the same information plus schema graph stats and index performance info.
    
    /// Find schemas missing artifacts for a specific language
    #[tool]
    async fn schemas_without_artifacts(
        &self,
        #[doc = "Target language: rust, typescript, python"]
        lang: String,
    ) -> Result<String> {
        let all_schemas: Vec<_> = self.graph.all_ids().collect();
        let with_artifacts: std::collections::HashSet<_> = self.graph
            .schemas_with_artifacts(&lang)
            .into_iter()
            .map(|(id, _)| id.clone())
            .collect();
        
        let mut missing: Vec<serde_json::Value> = all_schemas
            .iter()
            .filter(|id| !with_artifacts.contains(**id))
            .filter_map(|id| {
                let node = self.graph.get(id)?;
                Some(json!({
                    "id": id,
                    "kind": node.kind,
                    "title": node.title
                }))
            })
            .collect();
        
        // Sort by kind then id
        missing.sort_by(|a, b| {
            let a_kind = a.get("kind").and_then(|v| v.as_str()).unwrap_or("");
            let b_kind = b.get("kind").and_then(|v| v.as_str()).unwrap_or("");
            a_kind.cmp(b_kind).then_with(|| {
                a.get("id").and_then(|v| v.as_str()).unwrap_or("")
                    .cmp(&b.get("id").and_then(|v| v.as_str()).unwrap_or(""))
            })
        });
        
        Ok(serde_json::to_string_pretty(&json!({
            "lang": lang,
            "missing_count": missing.len(),
            "total_schemas": all_schemas.len(),
            "missing": missing
        }))?)
    }
    
    // ========== Graph-Based Artifact Analysis Tools ==========
    
    /// Find all artifacts that would be affected by changing a schema
    /// Uses graph traversal to find transitive dependents
    #[tool]
    async fn affected_artifacts(&self, id: String) -> Result<String> {
        let resolved_id = self.graph.resolve(&id)
            .cloned()
            .unwrap_or_else(|| id.clone());
        
        let affected = self.graph.affected_artifacts(&resolved_id);
        
        // Group by language
        let mut by_lang: std::collections::HashMap<String, Vec<serde_json::Value>> = std::collections::HashMap::new();
        for artifact in &affected {
            by_lang.entry(artifact.lang.clone()).or_default().push(json!({
                "file": artifact.file,
                "line": artifact.line,
                "type_name": artifact.type_name,
                "type_kind": artifact.type_kind
            }));
        }
        
        Ok(serde_json::to_string_pretty(&json!({
            "schema_id": resolved_id,
            "total_affected": affected.len(),
            "by_language": by_lang,
            "note": "These artifacts may need regeneration if the schema changes"
        }))?)
    }
    
    /// Find all schemas needed to generate a specific artifact
    /// Uses graph traversal to find transitive dependencies
    #[tool]
    async fn artifact_dependencies(
        &self,
        #[doc = "Artifact ID (e.g., 'rust:TenantId')"]
        artifact_id: String,
    ) -> Result<String> {
        let deps = self.graph.artifact_dependencies(&artifact_id);
        
        let schemas: Vec<_> = deps.iter().map(|node| {
            json!({
                "id": node.id,
                "title": node.title,
                "kind": node.kind
            })
        }).collect();
        
        Ok(serde_json::to_string_pretty(&json!({
            "artifact_id": artifact_id,
            "dependency_count": schemas.len(),
            "dependencies": schemas,
            "note": "All schemas that contribute to this artifact's generated code"
        }))?)
    }
    
    /// Find all artifacts in the same file as a given artifact
    #[tool]
    async fn colocated_artifacts(
        &self,
        #[doc = "Artifact ID (e.g., 'rust:TenantId')"]
        artifact_id: String,
    ) -> Result<String> {
        let colocated = self.graph.colocated_artifacts(&artifact_id);
        
        let artifacts: Vec<_> = colocated.iter().map(|a| {
            json!({
                "artifact_id": a.artifact_id(),
                "type_name": a.type_name,
                "type_kind": a.type_kind,
                "line": a.line
            })
        }).collect();
        
        // Get file info
        let file = self.graph.get_artifact_by_id(&artifact_id)
            .map(|a| a.file.to_string_lossy().to_string());
        
        Ok(serde_json::to_string_pretty(&json!({
            "artifact_id": artifact_id,
            "file": file,
            "colocated_count": artifacts.len(),
            "colocated": artifacts
        }))?)
    }
    
    /// Get all artifacts in a specific file
    #[tool]
    async fn file_artifacts(
        &self,
        #[doc = "File path (relative or absolute)"]
        file: String,
    ) -> Result<String> {
        use std::path::PathBuf;
        
        let path = PathBuf::from(&file);
        let artifacts = self.graph.get_file_artifacts(&path);
        
        let items: Vec<_> = artifacts.iter().map(|a| {
            let schema_id = self.graph.get_artifact_schema(&a.artifact_id());
            json!({
                "artifact_id": a.artifact_id(),
                "schema_id": schema_id,
                "type_name": a.type_name,
                "type_kind": a.type_kind,
                "line": a.line
            })
        }).collect();
        
        Ok(serde_json::to_string_pretty(&json!({
            "file": file,
            "artifact_count": items.len(),
            "artifacts": items
        }))?)
    }
    
    /// Get graph statistics including artifact indexes
    #[tool]
    async fn graph_stats(&self) -> Result<String> {
        let coverage = self.graph.artifact_coverage();
        
        Ok(serde_json::to_string_pretty(&json!({
            "schemas": {
                "total": self.graph.schema_count(),
                "edges": self.graph.edge_count(),
                "sccs": self.graph.scc_count()
            },
            "artifacts": {
                "total": self.graph.artifact_count(),
                "by_language": coverage.iter().map(|(lang, (count, total))| {
                    json!({
                        "lang": lang,
                        "count": count,
                        "coverage_pct": format!("{:.1}%", (*count as f64 / *total as f64) * 100.0)
                    })
                }).collect::<Vec<_>>()
            },
            "indexes": {
                "note": "All lookups are O(1) via HashMap indexes",
                "schema_to_artifacts": "O(1)",
                "artifact_to_schema": "O(1)",
                "file_to_artifacts": "O(1)",
                "lang_to_artifacts": "O(1)"
            }
        }))?)
    }
    
    // Note: lint_facets and lint_all_facets tools have been moved to xtask.
    // Use `cargo xtask schemas lint-facets` for schema facet linting.
}

// Helper: Check if all fields support Eq/Hash
fn is_eq_hash_safe(schema: &serde_json::Value) -> bool {
    if let Some(props) = schema.get("properties").and_then(|p| p.as_object()) {
        for (_, prop_schema) in props {
            // f64/number without format is not safe
            if prop_schema.get("type").and_then(|t| t.as_str()) == Some("number")
                && prop_schema.get("format").is_none()
            {
                return false;
            }
            // Generic object/Value is not safe
            if prop_schema.get("type").and_then(|t| t.as_str()) == Some("object")
                && prop_schema.get("properties").is_none()
            {
                return false;
            }
            // HashMap is not safe
            if prop_schema.get("additionalProperties").is_some() {
                return false;
            }
        }
    }
    true
}

// Helper: Normalize JSON for hashing (remove descriptions, titles, x-* extensions)
fn normalize_for_hash(v: &serde_json::Value) -> String {
    use std::collections::BTreeMap;
    
    fn normalize(v: &serde_json::Value) -> serde_json::Value {
        match v {
            serde_json::Value::Object(m) => {
                let normalized: BTreeMap<_, _> = m
                    .iter()
                    .filter(|(k, _)| *k != "description" && *k != "title" && !k.starts_with("x-"))
                    .map(|(k, v)| (k.clone(), normalize(v)))
                    .collect();
                serde_json::Value::Object(normalized.into_iter().collect())
            }
            serde_json::Value::Array(a) => {
                serde_json::Value::Array(a.iter().map(normalize).collect())
            }
            _ => v.clone(),
        }
    }
    
    serde_json::to_string(&normalize(v)).unwrap_or_default()
}

// Helper: Convert snake_case to PascalCase
fn pascal_case(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-')
        .map(|w| {
            let mut c = w.chars();
            c.next()
                .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_default()
        })
        .collect()
}

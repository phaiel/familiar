//! Impl module for provider_config types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ProviderConfig

// Methods: openai, anthropic, google, mock, with_api_key, with_base_url, with_organization, effective_base_url
impl ProviderConfig { pub fn openai (api_key : ApiKey) -> Self { Self { provider : AIProvider :: OpenAI , api_key : Some (api_key) , base_url : None , organization_id : None , api_version : None } } pub fn anthropic (api_key : ApiKey) -> Self { Self { provider : AIProvider :: Anthropic , api_key : Some (api_key) , base_url : None , organization_id : None , api_version : Some ("2023-06-01" . to_string ()) } } pub fn google (api_key : ApiKey) -> Self { Self { provider : AIProvider :: Google , api_key : Some (api_key) , base_url : None , organization_id : None , api_version : None } } pub fn mock () -> Self { Self { provider : AIProvider :: Mock , api_key : None , base_url : None , organization_id : None , api_version : None } } pub fn with_api_key (mut self , key : ApiKey) -> Self { self . api_key = Some (key) ; self } pub fn with_base_url (mut self , url : impl Into < String >) -> Self { self . base_url = Some (url . into ()) ; self } pub fn with_organization (mut self , org_id : impl Into < String >) -> Self { self . organization_id = Some (org_id . into ()) ; self } pub fn effective_base_url (& self) -> & str { self . base_url . as_deref () . unwrap_or_else (| | self . provider . default_base_url ()) } }

// Trait impl: Default
impl Default for ProviderConfig { fn default () -> Self { Self :: mock () } }


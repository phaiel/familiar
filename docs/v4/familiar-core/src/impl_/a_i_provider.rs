//! Impl module for a_i_provider types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for AIProvider

// Methods: as_str, display_name, api_key_env_var, default_base_url, requires_api_key
impl AIProvider { pub fn as_str (& self) -> & 'static str { match self { Self :: OpenAI => "openai" , Self :: Anthropic => "anthropic" , Self :: Google => "google" , Self :: Mock => "mock" , } } pub fn display_name (& self) -> & 'static str { match self { Self :: OpenAI => "OpenAI" , Self :: Anthropic => "Anthropic" , Self :: Google => "Google" , Self :: Mock => "Mock" , } } pub fn api_key_env_var (& self) -> Option < & 'static str > { match self { Self :: OpenAI => Some ("OPENAI_API_KEY") , Self :: Anthropic => Some ("ANTHROPIC_API_KEY") , Self :: Google => Some ("GOOGLE_API_KEY") , Self :: Mock => None , } } pub fn default_base_url (& self) -> & 'static str { match self { Self :: OpenAI => "https://api.openai.com/v1" , Self :: Anthropic => "https://api.anthropic.com/v1" , Self :: Google => "https://generativelanguage.googleapis.com/v1beta" , Self :: Mock => "http://localhost:0" , } } pub fn requires_api_key (& self) -> bool { ! matches ! (self , Self :: Mock) } }

// Trait impl: Display
impl std :: fmt :: Display for AIProvider { fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result { write ! (f , "{}" , self . display_name ()) } }


//! Impl module for observer_error types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ObserverError

// Methods: configuration, network, parse_error, is_retryable, provider
impl ObserverError { pub fn configuration (message : impl Into < String >) -> Self { Self :: Configuration { message : message . into () } } pub fn network (provider : AIProvider , message : impl Into < String > , status_code : Option < u16 >) -> Self { Self :: Network { provider , message : message . into () , status_code } } pub fn parse_error (message : impl Into < String > , raw : Option < String >) -> Self { Self :: ParseError { message : message . into () , raw_content : raw } } pub fn is_retryable (& self) -> bool { matches ! (self , Self :: Network { .. } | Self :: RateLimited { .. } | Self :: Timeout { .. }) } pub fn provider (& self) -> Option < AIProvider > { match self { Self :: Configuration { .. } | Self :: ParseError { .. } => None , Self :: InvalidResponse { provider , .. } | Self :: Network { provider , .. } | Self :: RateLimited { provider , .. } | Self :: ContentFiltered { provider , .. } | Self :: Timeout { provider , .. } => Some (* provider) , } } }

// Trait impl: Display
impl std :: fmt :: Display for ObserverError { fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result { match self { Self :: Configuration { message } => write ! (f , "Configuration error: {}" , message) , Self :: InvalidResponse { provider , message , .. } => write ! (f , "{} invalid response: {}" , provider , message) , Self :: Network { provider , message , status_code } => { if let Some (code) = status_code { write ! (f , "{} network error (HTTP {}): {}" , provider , code , message) } else { write ! (f , "{} network error: {}" , provider , message) } } Self :: RateLimited { provider , retry_after_ms } => { if let Some (ms) = retry_after_ms { write ! (f , "{} rate limited, retry after {}ms" , provider , ms) } else { write ! (f , "{} rate limited" , provider) } } Self :: ContentFiltered { provider , message } => write ! (f , "{} content filtered: {}" , provider , message) , Self :: ParseError { message , .. } => write ! (f , "Parse error: {}" , message) , Self :: Timeout { provider , timeout_ms } => write ! (f , "{} timeout after {}ms" , provider , timeout_ms) , } } }

// Trait impl: Error
impl std :: error :: Error for ObserverError { }


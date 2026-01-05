//! Impl module for communication_summary types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for CommunicationSummary

// Trait impl: Display
impl std :: fmt :: Display for CommunicationSummary { fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result { if self . compliant { writeln ! (f , "✅ Communication patterns compliant - all internal traffic uses Redpanda/Kafka") } else { writeln ! (f , "❌ Communication pattern violations found:") ? ; if self . http_in_workers > 0 { writeln ! (f , "  - HTTP clients in workers: {}" , self . http_in_workers) ? ; } if self . http_client_fields > 0 { writeln ! (f , "  - HTTP client struct fields: {}" , self . http_client_fields) ? ; } if self . kafka_bypasses > 0 { writeln ! (f , "  - Kafka bypass calls: {}" , self . kafka_bypasses) ? ; } writeln ! (f , "  Total: {} violations" , self . total_violations) } } }


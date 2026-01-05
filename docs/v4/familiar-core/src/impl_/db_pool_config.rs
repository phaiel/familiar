//! Impl module for db_pool_config types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for DbPoolConfig

// Methods: new, with_max_connections, with_min_connections
impl DbPoolConfig { pub fn new (connection : DbConnectionString) -> Self { Self { connection , max_connections : DbPoolSize :: new (10) . unwrap () , min_connections : DbPoolSize :: new (1) . unwrap () , connect_timeout_secs : 30 , idle_timeout_secs : 600 , } } pub fn with_max_connections (mut self , size : u32) -> Result < Self , String > { self . max_connections = DbPoolSize :: new (size) ? ; Ok (self) } pub fn with_min_connections (mut self , size : u32) -> Result < Self , String > { self . min_connections = DbPoolSize :: new (size) ? ; Ok (self) } }

// Trait impl: Default
impl Default for DbPoolConfig { fn default () -> Self { Self :: new (DbConnectionString :: default ()) } }


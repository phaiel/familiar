//! Impl module for entity_meta types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for EntityMeta

// Methods: new
impl < Id : Default > EntityMeta < Id > { # [doc = " Create new entity metadata with a default ID"] pub fn new (tenant_id : TenantId) -> Self { Self { id : Id :: default () , tenant_id , timestamps : Timestamps :: now () , } } }

// Methods: with_id, from_db, touch
impl < Id > EntityMeta < Id > { # [doc = " Create entity metadata with a specific ID"] pub fn with_id (id : Id , tenant_id : TenantId) -> Self { Self { id , tenant_id , timestamps : Timestamps :: now () , } } # [doc = " Create from database values"] pub fn from_db (id : Id , tenant_id : TenantId , created_at : chrono :: DateTime < chrono :: Utc > , updated_at : chrono :: DateTime < chrono :: Utc >) -> Self { Self { id , tenant_id , timestamps : Timestamps :: from_db (created_at , updated_at) , } } # [doc = " Update the modification timestamp"] pub fn touch (& mut self) { self . timestamps . touch () ; } }


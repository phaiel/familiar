//! Impl module for db_store_error types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for DbStoreError

// Methods: connection, operation, query, not_found
impl DbStoreError { pub fn connection (msg : impl Into < String >) -> Self { Self :: Connection { message : msg . into () } } pub fn operation (msg : impl Into < String >) -> Self { Self :: Operation { message : msg . into () } } pub fn query (msg : impl Into < String >) -> Self { Self :: Query { message : msg . into () , query : None } } pub fn not_found (entity_type : impl Into < String > , id : impl Into < String >) -> Self { Self :: NotFound { entity_type : entity_type . into () , id : id . into () , } } }

// Trait impl: Display
impl std :: fmt :: Display for DbStoreError { fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result { match self { Self :: Connection { message } => write ! (f , "Connection error: {}" , message) , Self :: Query { message , .. } => write ! (f , "Query error: {}" , message) , Self :: NotFound { entity_type , id } => write ! (f , "{} not found: {}" , entity_type , id) , Self :: Constraint { message } => write ! (f , "Constraint violation: {}" , message) , Self :: Serialization { message } => write ! (f , "Serialization error: {}" , message) , Self :: Transaction { message } => write ! (f , "Transaction error: {}" , message) , Self :: Migration { message } => write ! (f , "Migration error: {}" , message) , Self :: Operation { message } => write ! (f , "Operation error: {}" , message) , } } }

// Trait impl: Error
impl std :: error :: Error for DbStoreError { }

// Trait impl: From
impl From < sqlx :: Error > for DbStoreError { fn from (err : sqlx :: Error) -> Self { match err { sqlx :: Error :: RowNotFound => Self :: NotFound { entity_type : "Unknown" . to_string () , id : "Unknown" . to_string () , } , sqlx :: Error :: Database (db_err) => { if db_err . is_unique_violation () || db_err . is_foreign_key_violation () { Self :: Constraint { message : db_err . to_string () } } else { Self :: Query { message : db_err . to_string () , query : None } } } _ => Self :: Query { message : err . to_string () , query : None } , } } }

// Trait impl: From
impl From < sea_orm :: DbErr > for DbStoreError { fn from (err : sea_orm :: DbErr) -> Self { DbStoreError :: query (err . to_string ()) } }


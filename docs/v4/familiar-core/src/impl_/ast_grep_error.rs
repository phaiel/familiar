//! Impl module for ast_grep_error types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for AstGrepError

// Trait impl: Display
impl std :: fmt :: Display for AstGrepError { fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result { match self { AstGrepError :: CommandFailed (e) => write ! (f , "ast-grep command failed: {}" , e) , AstGrepError :: ParseError (e) => write ! (f , "Failed to parse ast-grep output: {}" , e) , } } }

// Trait impl: Error
impl std :: error :: Error for AstGrepError { }


//! Impl module for metadata types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for Metadata

// Methods: new, from_value, get, get_raw, set, remove, contains, is_empty, keys, as_value, into_value, merge
impl Metadata { # [doc = " Create new empty metadata"] pub fn new () -> Self { Self (serde_json :: json ! ({ })) } # [doc = " Create metadata from a JSON value"] pub fn from_value (value : serde_json :: Value) -> Self { if value . is_object () { Self (value) } else { Self :: new () } } # [doc = " Get a typed value by key"] pub fn get < T : DeserializeOwned > (& self , key : & str) -> Option < T > { self . 0 . get (key) . and_then (| v | serde_json :: from_value (v . clone ()) . ok ()) } # [doc = " Get a raw JSON value by key"] pub fn get_raw (& self , key : & str) -> Option < & serde_json :: Value > { self . 0 . get (key) } # [doc = " Set a value by key"] pub fn set < T : Serialize > (& mut self , key : & str , value : T) { if let Some (obj) = self . 0 . as_object_mut () { if let Ok (json_value) = serde_json :: to_value (value) { obj . insert (key . to_string () , json_value) ; } } } # [doc = " Remove a key"] pub fn remove (& mut self , key : & str) -> Option < serde_json :: Value > { self . 0 . as_object_mut () . and_then (| obj | obj . remove (key)) } # [doc = " Check if a key exists"] pub fn contains (& self , key : & str) -> bool { self . 0 . get (key) . is_some () } # [doc = " Check if metadata is empty"] pub fn is_empty (& self) -> bool { self . 0 . as_object () . map (| obj | obj . is_empty ()) . unwrap_or (true) } # [doc = " Get all keys"] pub fn keys (& self) -> Vec < & str > { self . 0 . as_object () . map (| obj | obj . keys () . map (| k | k . as_str ()) . collect ()) . unwrap_or_default () } # [doc = " Get the inner JSON value"] pub fn as_value (& self) -> & serde_json :: Value { & self . 0 } # [doc = " Convert to inner JSON value"] pub fn into_value (self) -> serde_json :: Value { self . 0 } # [doc = " Merge another metadata object (other values override self)"] pub fn merge (& mut self , other : & Metadata) { if let (Some (base) , Some (other_obj)) = (self . 0 . as_object_mut () , other . 0 . as_object ()) { for (key , value) in other_obj { base . insert (key . clone () , value . clone ()) ; } } } }

// Trait impl: Display
impl fmt :: Display for Metadata { fn fmt (& self , f : & mut fmt :: Formatter < '_ >) -> fmt :: Result { write ! (f , "{}" , self . 0) } }

// Trait impl: From
impl From < serde_json :: Value > for Metadata { fn from (value : serde_json :: Value) -> Self { Self :: from_value (value) } }

// Trait impl: PartialEq
impl PartialEq for Metadata { fn eq (& self , other : & Self) -> bool { self . 0 == other . 0 } }

// Trait impl: Eq
impl Eq for Metadata { }


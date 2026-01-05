//! Impl module for fast_walker types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for FastWalker

// Methods: new, collect_files, collect_rust_files, collect_typescript_files, collect_python_files
impl FastWalker { pub fn new (root : PathBuf) -> Self { Self { root , skip_dirs : vec ! ["node_modules" , "target" , ".next" , "dist" , "build" , ".git" , ".venv" , "venv" , "__pycache__" , "trashbin" , ".turbo" , "generated" ,] , } } # [doc = " Collect all source files using parallel walking"] # [doc = " Returns files grouped by type for efficient processing"] pub fn collect_files (& self) -> Vec < SourceFile > { let files = Mutex :: new (Vec :: new ()) ; let walker = WalkBuilder :: new (& self . root) . hidden (false) . git_ignore (true) . git_global (true) . git_exclude (true) . ignore (true) . parents (true) . build_parallel () ; let skip_dirs = & self . skip_dirs ; walker . run (| | { let files = & files ; let skip_dirs = skip_dirs ; Box :: new (move | entry | { use ignore :: WalkState ; let entry = match entry { Ok (e) => e , Err (_) => return WalkState :: Continue , } ; let path = entry . path () ; if let Some (name) = path . file_name () . and_then (| n | n . to_str ()) { if skip_dirs . contains (& name) { return WalkState :: Skip ; } } if ! entry . file_type () . map (| ft | ft . is_file ()) . unwrap_or (false) { return WalkState :: Continue ; } if let Some (ext) = path . extension () . and_then (| e | e . to_str ()) { if let Some (file_type) = FileType :: from_extension (ext) { files . lock () . unwrap () . push (SourceFile { path : path . to_path_buf () , file_type , }) ; } } WalkState :: Continue }) }) ; files . into_inner () . unwrap () } # [doc = " Collect files and return only Rust files"] pub fn collect_rust_files (& self) -> Vec < PathBuf > { self . collect_files () . into_iter () . filter (| f | f . file_type == FileType :: Rust) . map (| f | f . path) . collect () } # [doc = " Collect files and return only TypeScript files"] pub fn collect_typescript_files (& self) -> Vec < PathBuf > { self . collect_files () . into_iter () . filter (| f | f . file_type == FileType :: TypeScript) . map (| f | f . path) . collect () } # [doc = " Collect files and return only Python files"] pub fn collect_python_files (& self) -> Vec < PathBuf > { self . collect_files () . into_iter () . filter (| f | f . file_type == FileType :: Python) . map (| f | f . path) . collect () } }


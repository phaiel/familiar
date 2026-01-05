//! Impl module for media_store types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for MediaStore

// Methods: new, ensure_bucket, upload, get_presigned_url, delete
impl MediaStore { # [doc = " Create a new MediaStore instance"] pub async fn new (config : MediaStoreConfig) -> Self { let creds = aws_sdk_s3 :: config :: Credentials :: new (config . access_key , config . secret_key , None , None , "familiar" ,) ; let sdk_config = aws_config :: from_env () . region (Region :: new (config . region)) . endpoint_url (config . endpoint) . credentials_provider (creds) . load () . await ; let client = Client :: new (& sdk_config) ; Self { client , bucket : config . bucket , } } # [doc = " Initialize the bucket if it doesn't exist"] pub async fn ensure_bucket (& self) -> Result < () , DbStoreError > { match self . client . head_bucket () . bucket (& self . bucket) . send () . await { Ok (_) => Ok (()) , Err (_) => { self . client . create_bucket () . bucket (& self . bucket) . send () . await . map_err (| e | DbStoreError :: connection (e . to_string ())) ? ; Ok (()) } } } # [doc = " Upload binary data to the store"] pub async fn upload (& self , key : & str , data : Vec < u8 > , content_type : & str) -> Result < () , DbStoreError > { self . client . put_object () . bucket (& self . bucket) . key (key) . body (ByteStream :: from (data)) . content_type (content_type) . send () . await . map_err (| e | DbStoreError :: operation (e . to_string ())) ? ; Ok (()) } # [doc = " Generate a presigned URL for downloading"] pub async fn get_presigned_url (& self , key : & str , expires_in : Duration) -> Result < String , DbStoreError > { let presigning_config = PresigningConfig :: expires_in (expires_in) . map_err (| e | DbStoreError :: operation (e . to_string ())) ? ; let presigned_req = self . client . get_object () . bucket (& self . bucket) . key (key) . presigned (presigning_config) . await . map_err (| e | DbStoreError :: operation (e . to_string ())) ? ; Ok (presigned_req . uri () . to_string ()) } # [doc = " Delete an object"] pub async fn delete (& self , key : & str) -> Result < () , DbStoreError > { self . client . delete_object () . bucket (& self . bucket) . key (key) . send () . await . map_err (| e | DbStoreError :: operation (e . to_string ())) ? ; Ok (()) } }


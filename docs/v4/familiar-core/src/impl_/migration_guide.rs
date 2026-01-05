//! Impl module for migration_guide types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for MigrationGuide

// Methods: for_issue
impl MigrationGuide { # [doc = " Generate migration steps for a given issue"] pub fn for_issue (issue : & Issue) -> Vec < String > { match & issue . kind { IssueKind :: DirectHttpInWorker { component , .. } => { vec ! [format ! ("1. Remove {} from worker code" , component) , "2. Add EnvelopeProducer to your struct instead" . to_string () , "3. Emit commands via producer.send_command(&envelope)" . to_string () , "4. Create a Kafka consumer on the target service" . to_string () , "5. Process results via Kafka events" . to_string () ,] } IssueKind :: HttpClientInWorkerStruct { struct_name , .. } => { vec ! [format ! ("1. Remove http_client field from {}" , struct_name) , "2. Add envelope_producer: EnvelopeProducer field" . to_string () , "3. Update constructor to create EnvelopeProducer" . to_string () , "4. Replace HTTP calls with Kafka messages" . to_string () ,] } IssueKind :: InterServiceBypassKafka { target_service , method , .. } => { vec ! [format ! ("1. Remove direct {} call to {}" , method , target_service) , format ! ("2. Create {}Request envelope payload type" , target_service) , "3. Emit request via EnvelopeProducer" . to_string () , format ! ("4. Set up Kafka consumer in {}" , target_service) , "5. Return results via Kafka events" . to_string () ,] } _ => vec ! ["See documentation for migration guidance" . to_string ()] , } } }


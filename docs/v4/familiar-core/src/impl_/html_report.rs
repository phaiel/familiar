//! Impl module for html_report types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for HtmlReport

// Methods: from_analysis
impl HtmlReport { # [doc = " Create HTML report from analysis report"] pub fn from_analysis (report : & AnalysisReport , json_content : & str) -> Self { let (core_issues , services_issues) : (Vec < _ > , Vec < _ >) = report . issues . iter () . partition (| i | is_core_file (& i . file . display () . to_string ())) ; let core_tab = build_tab_data ("Core" , "📦" , "familiar-core, familiar-contracts, familiar-primitives - Source of truth" , & core_issues ,) ; let services_tab = build_tab_data ("Services" , "🚀" , "familiar-api, familiar-worker, windmill - Consumers of core" , & services_issues ,) ; let escaped_json = json_content . replace ('\\' , "\\\\") . replace ('`' , "\\`") . replace ("${" , "\\${") ; let total_errors = core_tab . error_count + services_tab . error_count ; let total_warnings = core_tab . warning_count + services_tab . warning_count ; let total_infos = core_tab . info_count + services_tab . info_count ; Self { timestamp : chrono_lite_now () , stats_cards : build_stats_cards (& report . stats , total_errors , total_warnings , total_infos) , core_tab , services_tab , escaped_json , } } }


//! Security Configuration Schema
//!
//! Defines constants and types for security-related configuration.

/// API key variable paths (Windmill variables)
pub struct WindmillSecrets {
    pub anthropic: &'static str,
    pub openai: &'static str,
    pub google: &'static str,
}

pub const WINDMILL_SECRETS: WindmillSecrets = WindmillSecrets {
    anthropic: "u/phaiel/ANTHROPIC_API_KEY",
    openai: "u/phaiel/OPENAI_API_KEY",
    google: "u/phaiel/GOOGLE_API_KEY",
};






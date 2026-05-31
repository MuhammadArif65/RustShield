use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod keygen;
pub mod validator;

pub use keygen::{generate_keypair, sign_license};
pub use validator::validate_license_secure;

/// The core license payload data.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LicenseData {
    /// Unique identifier for the game
    pub game_id: String,
    /// Hardware fingerprint of the buyer's machine
    pub hardware_id: String,
    /// Unix timestamp when the license was issued
    pub issued_at: u64,
    /// Unix timestamp when the license expires, or None for perpetual
    pub expires_at: Option<u64>,
    /// The edition of the game (e.g., "standard", "deluxe", "beta")
    pub edition: String,
    /// Custom metadata provided by the developer
    pub metadata: HashMap<String, String>,
}

//

use crate::error::{Result, FerrumWardError};
use crate::fingerprint::{get_hwid_profile, HwidProfile};
use crate::license::LicenseData;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use std::time::{SystemTime, UNIX_EPOCH};

/// Validates a license string against the given public key, game ID, and hardware fingerprint.
///
/// # Errors
/// Returns an appropriate `FerrumWardError` such as `InvalidSignature`, `HardwareMismatch`,
/// `LicenseExpired`, `GameIdMismatch`, or `MalformedLicense`. Note that for high-level APIs,
/// these should be mapped to `TamperDetected` to prevent leaking the exact validation step that failed.
pub(crate) fn validate_license(
    license_str: &str,
    public_key: &VerifyingKey,
    expected_game_id: &str,
) -> Result<LicenseData> {
    // 1. Split the license string into payload and signature parts
    let parts: Vec<&str> = license_str.split('.').collect();
    if parts.len() != 2 {
        return Err(FerrumWardError::MalformedLicense);
    }

    let payload_b64 = parts[0];
    let sig_b64 = parts[1];

    // 2. Decode base64 parts
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| FerrumWardError::MalformedLicense)?;
    let sig_bytes = URL_SAFE_NO_PAD
        .decode(sig_b64)
        .map_err(|_| FerrumWardError::MalformedLicense)?;

    // 3. Verify signature
    let signature =
        Signature::from_slice(&sig_bytes).map_err(|_| FerrumWardError::MalformedLicense)?;

    public_key
        .verify(&payload_bytes, &signature)
        .map_err(|_| FerrumWardError::InvalidSignature)?;

    // 4. Parse the payload into LicenseData
    let license_data: LicenseData =
        serde_json::from_slice(&payload_bytes).map_err(|_| FerrumWardError::MalformedLicense)?;

    // 5. Verify Game ID
    if license_data.game_id != expected_game_id {
        return Err(FerrumWardError::GameIdMismatch);
    }

    // 6. Verify Hardware ID using Fuzzy Matching
    let current_profile = get_hwid_profile()?.to_hashed();

    // Decode expected profile
    let expected_json = String::from_utf8(
        URL_SAFE_NO_PAD
            .decode(&license_data.hardware_id)
            .map_err(|_| FerrumWardError::HardwareMismatch)?,
    )
    .map_err(|_| FerrumWardError::HardwareMismatch)?;

    let expected_profile: HwidProfile =
        serde_json::from_str(&expected_json).map_err(|_| FerrumWardError::HardwareMismatch)?;

    let score = expected_profile.match_score(&current_profile);
    println!("EXPECTED HWID: {:?}", expected_profile);
    println!("CURRENT HWID: {:?}", current_profile);
    println!("SCORE: {}", score);

    // Allow up to 3 components to change (must match at least 4 out of 7)
    if score < 4 {
        return Err(FerrumWardError::HardwareMismatch);
    }

    // 7. Verify Expiry
    if let Some(expiry) = license_data.expires_at {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| FerrumWardError::TamperDetected)?
            .as_secs();

        if current_time >= expiry {
            return Err(FerrumWardError::LicenseExpired);
        }
    }

    Ok(license_data)
}

/// A secure wrapper around `validate_license` that maps any validation failure
/// (e.g. malformed license, hardware mismatch, signature failure) to a generic
/// `TamperDetected` error, to prevent leaking specifics of the validation failure.
pub fn validate_license_secure(
    license_str: &str,
    public_key: &VerifyingKey,
    expected_game_id: &str,
) -> Result<LicenseData> {
    match validate_license(license_str, public_key, expected_game_id) {
        Ok(data) => Ok(data),
        Err(_) => Err(FerrumWardError::TamperDetected),
    }
}

//

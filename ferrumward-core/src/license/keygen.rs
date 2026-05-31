use crate::error::{Result, FerrumWardError};
use crate::license::LicenseData;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;

/// Generates an Ed25519 keypair for license signing.
/// WARNING: Do not clone `SigningKey` directly in application code to prevent side-channel attacks or key leakage.
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let mut csprng = OsRng;
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

/// Signs a LicenseData object and returns a formatted license string.
/// Format: `base64url(json_payload).base64url(ed25519_signature)`
pub fn sign_license(license_data: &LicenseData, signing_key: &SigningKey) -> Result<String> {
    let payload = serde_json::to_string(license_data)
        .map_err(|e| FerrumWardError::CryptoError(format!("Serialization error: {}", e)))?;

    let signature = signing_key.sign(payload.as_bytes());

    let payload_b64 = URL_SAFE_NO_PAD.encode(payload.as_bytes());
    let sig_b64 = URL_SAFE_NO_PAD.encode(signature.to_bytes());

    Ok(format!("{}.{}", payload_b64, sig_b64))
}

//

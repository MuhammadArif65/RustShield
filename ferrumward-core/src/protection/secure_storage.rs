use crate::error::{Result, FerrumWardError};

#[cfg(feature = "crypto")]
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
#[cfg(feature = "crypto")]
use rand::Rng;

/// Encrypts save data or assets using a hardware-bound key.
/// Requires the `crypto` feature.
#[cfg(feature = "crypto")]
pub fn encrypt_secure_asset(data: &[u8], encryption_key: &[u8]) -> Result<Vec<u8>> {
    if encryption_key.len() != 32 {
        return Err(FerrumWardError::TamperDetected); // Key must be 32 bytes
    }

    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(encryption_key);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes); // 96-bits

    let mut ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|_| FerrumWardError::TamperDetected)?;

    // Prepend nonce to ciphertext
    let mut final_data = nonce_bytes.to_vec();
    final_data.append(&mut ciphertext);

    Ok(final_data)
}

/// Decrypts save data or assets using a hardware-bound key.
/// Requires the `crypto` feature.
#[cfg(feature = "crypto")]
pub fn decrypt_secure_asset(encrypted_data: &[u8], encryption_key: &[u8]) -> Result<Vec<u8>> {
    if encryption_key.len() != 32 || encrypted_data.len() < 28 {
        return Err(FerrumWardError::TamperDetected);
    }

    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(encryption_key);
    let cipher = Aes256Gcm::new(key);

    let nonce = Nonce::from_slice(&encrypted_data[0..12]);
    let ciphertext = &encrypted_data[12..];

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| FerrumWardError::TamperDetected)?;

    Ok(plaintext)
}

//

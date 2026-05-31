use crate::error::{FerrumWardError, Result};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};

pub mod asset_crypto;

pub use asset_crypto::*;

pub struct AesGcm256 {
    cipher: Aes256Gcm,
}

impl AesGcm256 {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        Self { cipher }
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; 12 bytes
        let encrypted = self
            .cipher
            .encrypt(&nonce, data)
            .map_err(|_| FerrumWardError::CryptoError("encryption failed".to_string()))?;

        let mut output = nonce.to_vec();
        output.extend_from_slice(&encrypted);
        Ok(output)
    }

    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        if encrypted_data.len() < 12 {
            return Err(FerrumWardError::CryptoError("data too short".to_string()));
        }
        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let ciphertext = &encrypted_data[12..];

        let decrypted = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| FerrumWardError::CryptoError("decryption failed".to_string()))?;

        Ok(decrypted)
    }
}

//

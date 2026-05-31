use crate::error::{Result, FerrumWardError};
use sha2::{Digest, Sha256};
use std::env;
use std::fs::File;
use std::io::Read;
use std::sync::OnceLock;

static ORIGINAL_HASH: OnceLock<String> = OnceLock::new();

/// Initializes the self-check mechanism by hashing the current executable on disk.
/// Note: Advanced in-memory `.text` section hashing requires OS-specific API access (e.g. `VirtualQuery` on Windows).
/// For zero-dependency pure Rust, we rely on the on-disk integrity as a baseline.
pub fn init_self_check() -> Result<()> {
    let exe_path = env::current_exe().map_err(|_| FerrumWardError::TamperDetected)?;

    let mut file = File::open(&exe_path).map_err(|_| FerrumWardError::TamperDetected)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|_| FerrumWardError::TamperDetected)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let hash = format!("{:x}", hasher.finalize());
    let _ = ORIGINAL_HASH.set(hash);

    Ok(())
}

/// Periodically called to verify that the executable has not been replaced or modified.
pub fn verify_self_check() -> Result<()> {
    let original = ORIGINAL_HASH.get().ok_or(FerrumWardError::TamperDetected)?;

    let exe_path = env::current_exe().map_err(|_| FerrumWardError::TamperDetected)?;

    let mut file = match File::open(&exe_path) {
        Ok(f) => f,
        Err(_) => return Err(FerrumWardError::TamperDetected), // File is missing/locked?
    };

    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|_| FerrumWardError::TamperDetected)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let current_hash = format!("{:x}", hasher.finalize());

    if &current_hash != original {
        return Err(FerrumWardError::TamperDetected);
    }

    Ok(())
}

//

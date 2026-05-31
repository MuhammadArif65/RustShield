use crate::error::{Result, FerrumWardError};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

static BASELINE_HASH: OnceLock<String> = OnceLock::new();

/// Initializes the memory integrity baseline.
/// This should be called once during startup.
pub fn init_memory_integrity() -> Result<()> {
    let hash = compute_text_section_hash()?;
    let _ = BASELINE_HASH.set(hash);
    Ok(())
}

/// Verifies that the memory hasn't been patched.
pub fn verify_memory_integrity() -> Result<()> {
    let baseline = match BASELINE_HASH.get() {
        Some(h) => h,
        None => return Ok(()), // Not initialized
    };

    let current = compute_text_section_hash()?;
    if current != *baseline {
        return Err(FerrumWardError::TamperDetected);
    }
    Ok(())
}

/// Computes a hash of critical functions in memory to detect inline hooking or byte patching.
fn compute_text_section_hash() -> Result<String> {
    // To remain zero-dependency and minimal, we hash the first 16 bytes
    // of some critical functions in our own library.
    // If a cheat tries to place a JMP (hook) here, the hash will change.

    let fn_ptr1 = crate::protection::protect as *const u8;
    let fn_ptr2 = verify_memory_integrity as *const u8;

    let mut hasher = Sha256::new();

    // SAFETY: [Rule 2 Exception] Reading function pointers to compute memory checksums.
    // We only read 16 bytes from a known valid code section.
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    unsafe {
        let slice1 = std::slice::from_raw_parts(fn_ptr1, 16);
        hasher.update(slice1);

        let slice2 = std::slice::from_raw_parts(fn_ptr2, 16);
        hasher.update(slice2);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

//

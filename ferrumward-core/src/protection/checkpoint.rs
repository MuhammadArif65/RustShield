use crate::error::{FerrumWardError, Result};
#[cfg(feature = "hardware-binding")]
use crate::fingerprint::get_hardware_id;
use crate::protection::integrity::get_active_state;

/// Macro to insert a checkpoint in critical code sections.
/// It silently calls `verify_checkpoint()` to re-verify integrity without overhead.
#[macro_export]
macro_rules! ferrumward_checkpoint {
    () => {
        $crate::protection::checkpoint::verify_checkpoint()
    };
}

/// Manually verify all checkpoints. Returns a generic TamperDetected on failure
/// to avoid leaking which specific check failed.
pub fn verify_checkpoint() -> Result<()> {
    let state = match get_active_state() {
        Some(s) => s,
        None => return Err(crate::error::FerrumWardError::TamperDetected),
    };

    // 1. Re-verify HWID (fast, < 1ms)
    #[cfg(feature = "hardware-binding")]
    match get_hardware_id() {
        Ok(current_hwid) if current_hwid == state.original_hwid => {}
        _ => return trigger_failure(&state.config),
    }

    // 2. Check canary values intact
    #[cfg(feature = "canary")]
    for canary in &state.canaries {
        if !canary.check() {
            return trigger_failure(&state.config);
        }
    }

    // 3. (Planned) Check timestamp for tampering
    // 4. (Planned) Re-hash critical files

    Ok(())
}

fn trigger_failure(config: &crate::protection::integrity::ProtectionConfig) -> Result<()> {
    if let Some(ref cb) = config.on_failure {
        cb(FerrumWardError::TamperDetected);
    }
    Err(FerrumWardError::TamperDetected)
}

//

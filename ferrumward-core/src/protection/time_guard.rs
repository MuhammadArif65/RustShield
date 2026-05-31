use crate::error::{FerrumWardError, Result};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};

/// Global state to track time continuously
static TIME_GUARD_STATE: OnceLock<Mutex<TimeGuardState>> = OnceLock::new();

#[derive(Debug)]
struct TimeGuardState {
    base_system_time: SystemTime,
    base_instant: Instant,
}

/// Initializes the Time Guard. Should be called exactly once at startup.
pub fn init_time_guard() -> Result<()> {
    let state = TimeGuardState {
        base_system_time: SystemTime::now(),
        base_instant: Instant::now(),
    };

    if TIME_GUARD_STATE.set(Mutex::new(state)).is_err() {
        // Already initialized
    }
    Ok(())
}

/// Verifies that the system clock has not been manipulated.
/// Returns an error if the monotonic elapsed time and system elapsed time
/// differs by more than a given tolerance (e.g., 30 seconds).
pub fn check_time_tampering() -> Result<()> {
    let mut state = match TIME_GUARD_STATE.get() {
        Some(s) => s.lock().unwrap(),
        None => return Err(FerrumWardError::TamperDetected),
    };

    let current_instant = Instant::now();
    let current_system = SystemTime::now();

    let elapsed_monotonic = current_instant.duration_since(state.base_instant);

    // Check if system time went backwards
    let drift = match current_system.duration_since(state.base_system_time) {
        Ok(elapsed_system) => {
            if elapsed_system > elapsed_monotonic {
                let d = elapsed_system - elapsed_monotonic;
                // If system time jumped forward by more than 1 hour, assume hibernation/suspend
                // and reset the baseline to avoid false positives.
                if d > Duration::from_secs(3600) {
                    state.base_instant = current_instant;
                    state.base_system_time = current_system;
                    return Ok(());
                }
                d
            } else {
                let d = elapsed_monotonic - elapsed_system;
                // If monotonic time is much faster than system time, it's definitely a speedhack
                if d > Duration::from_secs(10) {
                    return Err(FerrumWardError::TamperDetected);
                }
                d
            }
        }
        Err(e) => {
            // System time is before base_system_time. The drift is the time
            // that should have elapsed (monotonic) plus how far back it went.
            elapsed_monotonic + e.duration()
        }
    };

    // Allow up to 30 seconds of general drift for minor clock syncs
    if drift > Duration::from_secs(30) {
        return Err(FerrumWardError::TamperDetected);
    }

    Ok(())
}

//

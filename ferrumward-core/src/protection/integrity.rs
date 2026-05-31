use crate::error::{FerrumWardError, Result};
#[cfg(feature = "hardware-binding")]
use crate::fingerprint::get_hardware_id;
#[cfg(feature = "anti-debug")]
use crate::protection::anti_debug::assert_no_debugger;
use crate::protection::anti_dump::erase_headers_from_memory;
use crate::protection::anti_inject::assert_no_injected_modules;
use crate::protection::anti_suspend::start_anti_suspend_watchdog;
#[cfg(feature = "anti-vm")]
use crate::protection::anti_vm::assert_no_virtual_machine;
#[cfg(feature = "canary")]
use crate::protection::canary::CanaryGuard;
use crate::protection::chaotic_thread::start_chaotic_hive_mind;
use crate::protection::decoy_honeypot::DecoyHoneypot;
use crate::protection::hw_breakpoint::assert_no_hardware_breakpoints;
use crate::protection::mem_integrity::{init_memory_integrity, verify_memory_integrity};
use crate::protection::parent_process::assert_valid_parent_process;
use crate::protection::self_check::{init_self_check, verify_self_check};
use crate::protection::time_guard::{check_time_tampering, init_time_guard};
use rand::Rng;
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::Duration;

/// Configuration for the FerrumWard protection system.
pub struct ProtectionConfig {
    pub game_id: String,
    pub public_key: Vec<u8>,
    pub license: Option<String>,
    pub manifest_path: Option<std::path::PathBuf>,
    pub anti_debug: bool,
    pub anti_vm: bool,
    /// A generic failure callback. Do NOT reveal specific reasons here.
    pub on_failure: Option<Box<dyn Fn(FerrumWardError) + Send + Sync>>,
}

/// Global state to support `ferrumward_checkpoint!` macros.
pub struct ActiveProtectionState {
    pub config: Arc<ProtectionConfig>,
    #[cfg(feature = "canary")]
    pub canaries: Vec<CanaryGuard>,
    #[cfg(feature = "hardware-binding")]
    pub original_hwid: String,
    pub honeypot: DecoyHoneypot,
}

static ACTIVE_STATE: OnceLock<ActiveProtectionState> = OnceLock::new();

/// Entry point to initialize FerrumWard protection.
/// Must be called as early as possible during game startup.
pub fn protect(config: ProtectionConfig) -> Result<()> {
    // Layer 1: Anti-Debug
    #[cfg(feature = "anti-debug")]
    if config.anti_debug {
        if let Err(e) = assert_no_debugger() {
            trigger_failure(&config, e)?;
        }
    }

    // Layer 2: Anti-VM
    #[cfg(feature = "anti-vm")]
    if config.anti_vm {
        if let Err(e) = assert_no_virtual_machine() {
            trigger_failure(&config, e)?;
        }
    }

    // Layer 3: Anti-Injection & Parent Process
    if let Err(e) = assert_no_injected_modules() {
        trigger_failure(&config, e)?;
    }
    if let Err(e) = assert_valid_parent_process() {
        trigger_failure(&config, e)?;
    }

    // Initialize Canaries, Time Guard, and Self-Check
    #[cfg(feature = "canary")]
    let mut canaries = Vec::new();
    #[cfg(feature = "canary")]
    for _ in 0..5 {
        canaries.push(CanaryGuard::new());
    }

    if init_time_guard().is_err() {
        return Err(FerrumWardError::TamperDetected);
    }

    if init_self_check().is_err() {
        return Err(FerrumWardError::TamperDetected);
    }

    if init_memory_integrity().is_err() {
        return Err(FerrumWardError::TamperDetected);
    }

    // Erase headers from memory to prevent dumping
    let _ = erase_headers_from_memory(); // Ignore errors on unsupported platforms

    // Initialize state for background checks
    #[cfg(feature = "hardware-binding")]
    let hwid = match get_hardware_id() {
        Ok(id) => id,
        Err(e) => return trigger_failure(&config, e),
    };

    let config_arc = Arc::new(config);

    // Start Anti-Suspend watchdog (e.g. 2000ms threshold)
    let config_clone = config_arc.clone();
    let _watchdog_handle = start_anti_suspend_watchdog(
        2000,
        Some(Arc::new(Box::new(move |_err| {
            if let Some(ref cb) = config_clone.on_failure {
                cb(FerrumWardError::TamperDetected);
            }
        }))),
    );

    let state = ActiveProtectionState {
        config: config_arc.clone(),
        #[cfg(feature = "canary")]
        canaries,
        #[cfg(feature = "hardware-binding")]
        original_hwid: hwid,
        honeypot: DecoyHoneypot::new(),
    };

    // Store state (fail if already set)
    if ACTIVE_STATE.set(state).is_err() {
        return Err(FerrumWardError::TamperDetected);
    }

    // Start background loop (standard linear thread)
    start_protection_loop(config_arc.clone());

    // Start chaotic hive-mind threading
    let config_clone2 = config_arc.clone();
    let running_flag = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let check_fn = Arc::new(Box::new(|| {
        // Quick chaotic check (e.g., honeypot validation)
        if let Some(s) = ACTIVE_STATE.get() {
            s.honeypot.verify()?;
        }
        Ok(())
    }) as Box<dyn Fn() -> Result<()> + Send + Sync>);

    let failure_cb = Arc::new(Box::new(move |_err: FerrumWardError| {
        if let Some(ref cb) = config_clone2.on_failure {
            cb(FerrumWardError::TamperDetected);
        }
    }) as Box<dyn Fn(FerrumWardError) + Send + Sync>);

    start_chaotic_hive_mind(running_flag, check_fn, failure_cb);

    Ok(())
}

fn trigger_failure(config: &ProtectionConfig, _real_error: FerrumWardError) -> Result<()> {
    if let Some(ref cb) = config.on_failure {
        cb(FerrumWardError::TamperDetected);
    }
    // Return the generic error to caller as well
    Err(FerrumWardError::TamperDetected)
}

pub fn get_active_state() -> Option<&'static ActiveProtectionState> {
    ACTIVE_STATE.get()
}

/// Starts the background thread that periodically re-verifies integrity.
fn start_protection_loop(config: Arc<ProtectionConfig>) {
    let config_thread = config.clone();
    let builder_res = thread::Builder::new()
        .name("ferrumward_main".into())
        .spawn(move || {
            loop {
                // 3. Time tampering check
                if check_time_tampering().is_err() {
                    if let Some(ref cb) = config_thread.on_failure {
                        cb(FerrumWardError::TamperDetected);
                    }
                    break;
                }

                // 4. Process self-check
                if verify_self_check().is_err() {
                    if let Some(ref cb) = config_thread.on_failure {
                        cb(FerrumWardError::TamperDetected);
                    }
                    break;
                }

                if crate::protection::scan_for_rwx_memory().is_err() {
                    if let Some(ref cb) = config_thread.on_failure {
                        cb(FerrumWardError::TamperDetected);
                    }
                    break;
                }

                // 5. Memory integrity check
                if verify_memory_integrity().is_err() {
                    if let Some(ref cb) = config_thread.on_failure {
                        cb(FerrumWardError::TamperDetected);
                    }
                    break;
                }

                // Sleep for a random interval between 10 and 60 seconds to prevent predictable timing attacks
                let wait_secs = 10 + (rand::thread_rng().gen::<u64>() % 50);
                thread::sleep(Duration::from_secs(wait_secs));

                let mut checks: Vec<fn() -> Result<()>> = Vec::new();

                #[cfg(feature = "hardware-binding")]
                checks.push(verify_hwid_silent);

                #[cfg(feature = "canary")]
                checks.push(verify_canaries_silent);

                #[cfg(feature = "anti-debug")]
                if config_thread.anti_debug {
                    checks.push(assert_no_debugger);
                }

                checks.push(assert_no_injected_modules);
                checks.push(assert_valid_parent_process);
                checks.push(assert_no_hardware_breakpoints);
                checks.push(|| {
                    if let Some(s) = ACTIVE_STATE.get() {
                        s.honeypot.verify()
                    } else {
                        Ok(())
                    }
                });

                let result = if !checks.is_empty() {
                    let check_id = rand::thread_rng().gen::<usize>() % checks.len();
                    checks[check_id]()
                } else {
                    Ok(())
                };

                if result.is_err() {
                    if let Some(ref cb) = config_thread.on_failure {
                        cb(FerrumWardError::TamperDetected);
                    }
                    break;
                }
            }
        });

    if builder_res.is_err() {
        if let Some(ref cb) = config.on_failure {
            cb(FerrumWardError::TamperDetected);
        }
    }
}

#[cfg(feature = "hardware-binding")]
fn verify_hwid_silent() -> Result<()> {
    let state = match ACTIVE_STATE.get() {
        Some(s) => s,
        None => return Ok(()),
    };

    match get_hardware_id() {
        Ok(current) if current == state.original_hwid => Ok(()),
        _ => Err(FerrumWardError::TamperDetected),
    }
}

#[cfg(feature = "canary")]
fn verify_canaries_silent() -> Result<()> {
    let state = match ACTIVE_STATE.get() {
        Some(s) => s,
        None => return Ok(()),
    };

    for canary in &state.canaries {
        if !canary.check() {
            return Err(FerrumWardError::TamperDetected);
        }
    }
    Ok(())
}

//

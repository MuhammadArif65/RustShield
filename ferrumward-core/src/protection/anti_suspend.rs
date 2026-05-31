#![allow(clippy::type_complexity)]
use crate::error::FerrumWardError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Starts a background watchdog thread that monitors process suspension.
/// If the process is suspended (e.g., by Cheat Engine or a debugger) for more than the threshold,
/// it will trigger a tamper detection failure.
pub type OnFailureCallback = Arc<Box<dyn Fn(FerrumWardError) + Send + Sync>>;

pub fn start_anti_suspend_watchdog(
    threshold_millis: u64,
    on_failure: Option<OnFailureCallback>,
) -> Arc<AtomicBool> {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    if let Err(e) = thread::Builder::new()
        .name("anti_suspend".to_string())
        .spawn(move || {
            let mut last_check = Instant::now();
            let sleep_dur = Duration::from_millis(500);

            while running_clone.load(Ordering::Relaxed) {
                thread::sleep(sleep_dur);

                let elapsed = last_check.elapsed().as_millis() as u64;

                // Expected elapsed is sleep_dur (~500ms).
                // If it's significantly larger (e.g. 500ms + threshold), the thread was suspended.
                if elapsed > (sleep_dur.as_millis() as u64 + threshold_millis) {
                    // Tamper detected!
                    if let Some(ref cb) = on_failure {
                        cb(FerrumWardError::TamperDetected);
                    }
                    break;
                }

                last_check = Instant::now();
            }
        })
    {
        eprintln!("Failed to spawn anti_suspend thread: {}", e);
    }

    running
}

//

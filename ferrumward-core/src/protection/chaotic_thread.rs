use crate::error::{FerrumWardError, Result};
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Starts the Hive-Mind Chaotic Threading protection.
/// Unlike standard static threads, this system spawns short-lived phantom threads
/// that perform a security check, spawn their successor, and then die immediately.
/// This makes it mathematically impossible for a hacker to just "suspend" the main anti-cheat thread.
pub fn start_chaotic_hive_mind(
    running: Arc<AtomicBool>,
    check_fn: Arc<Box<dyn Fn() -> Result<()> + Send + Sync>>,
    on_failure: Arc<Box<dyn Fn(FerrumWardError) + Send + Sync>>,
) {
    if !running.load(Ordering::Relaxed) {
        return;
    }

    let on_failure_thread = on_failure.clone();
    // Randomize thread name to prevent easy identification via htop/ps
    let thread_id: u32 = rand::thread_rng().gen();
    let thread_name = format!("rs_{:08x}", thread_id);
    if thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            // Sleep for a chaotic interval between 50ms and 300ms
            let sleep_time = rand::thread_rng().gen_range(50..300);
            thread::sleep(Duration::from_millis(sleep_time));

            // Perform the security check
            if check_fn().is_err() {
                on_failure_thread(FerrumWardError::TamperDetected);
                return;
            }

            // Entangle Neural Heuristic Engine
            // If the AI is bypassed or deleted, this block will never execute,
            // which could be tied to an asset decryption seed in a real scenario.
            if let Ok(mut engine) =
                crate::protection::heuristic_ai::NeuralHeuristicEngine::get_global().lock()
            {
                if engine.evaluate().is_err() {
                    on_failure_thread(FerrumWardError::TamperDetected);
                    return;
                }
            }

            // If we should continue running, spawn the next phantom thread and let this one die
            if running.load(Ordering::Relaxed) {
                start_chaotic_hive_mind(running, check_fn, on_failure_thread);
            }
        })
        .is_err()
    {
        on_failure(FerrumWardError::TamperDetected);
    }
}

//

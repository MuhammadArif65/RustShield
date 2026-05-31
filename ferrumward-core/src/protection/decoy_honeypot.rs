use crate::error::{Result, FerrumWardError};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

/// Highly attractive but completely fake variables to trap memory scanners like Cheat Engine.
/// The game engine must NEVER modify these variables. If they change, we know a cheater
/// is manipulating memory.
pub struct DecoyHoneypot {
    player_health: AtomicI32,
    player_gold: AtomicI32,
    infinite_ammo: AtomicBool,
    god_mode: AtomicBool,
    time_bomb_triggered: AtomicBool,
}

impl Default for DecoyHoneypot {
    fn default() -> Self {
        Self {
            player_health: AtomicI32::new(100),
            player_gold: AtomicI32::new(999),
            infinite_ammo: AtomicBool::new(false),
            god_mode: AtomicBool::new(false),
            time_bomb_triggered: AtomicBool::new(false),
        }
    }
}

impl DecoyHoneypot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Verifies that no external program has modified the honeypot variables.
    pub fn verify(&self) -> Result<()> {
        if self.time_bomb_triggered.load(Ordering::Relaxed) {
            // Delay the crash slightly to confuse the attacker if called randomly,
            // or just return TamperDetected which breaks the protection loop.
            return Err(FerrumWardError::TamperDetected);
        }
        if self.player_health.load(Ordering::Relaxed) != 100 {
            return Err(FerrumWardError::TamperDetected);
        }
        if self.player_gold.load(Ordering::Relaxed) != 999 {
            return Err(FerrumWardError::TamperDetected);
        }
        if self.infinite_ammo.load(Ordering::Relaxed) {
            return Err(FerrumWardError::TamperDetected);
        }
        if self.god_mode.load(Ordering::Relaxed) {
            return Err(FerrumWardError::TamperDetected);
        }

        Ok(())
    }

    /// Called by the FFI decoy APIs when an attacker tries to call "disable_security" functions.
    pub fn trigger_time_bomb(&self) {
        // Set the bomb flag. The next verify loop will fail.
        self.time_bomb_triggered.store(true, Ordering::SeqCst);
    }
}

//

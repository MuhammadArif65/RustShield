#![allow(warnings)]
use bevy::prelude::*;
use ferrumward_core::protection::{protect, ProtectionConfig};
use ferrumward_core::ferrumward_checkpoint;
use std::sync::{Arc, Mutex};

/// A Bevy Plugin that integrates FerrumWard anti-piracy protections.
pub struct FerrumWardPlugin {
    config: Arc<Mutex<Option<ProtectionConfig>>>,
}

impl FerrumWardPlugin {
    /// Creates a new FerrumWardPlugin with the given configuration.
    pub fn new(config: ProtectionConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(Some(config))),
        }
    }
}

impl Plugin for FerrumWardPlugin {
    fn build(&self, app: &mut App) {
        if let Ok(mut lock) = self.config.lock() {
            if let Some(config) = lock.take() {
                // Initialize the core protection engine.
                // It will spawn a background thread automatically.
                match protect(config) {
                    Ok(_) => {
                        // Register a system that runs every frame or conditionally to trigger checkpoints
                        app.add_systems(Update, ferrumward_bevy_checkpoint_system);
                    }
                    Err(_e) => {
                        // If the initial protection fails (e.g. invalid license, tampered files),
                        // the `on_failure` callback inside the config has already been triggered by `protect`.
                        // We do nothing else here to avoid leaking information.
                    }
                }
            }
        }
    }
}

/// A Bevy system that periodically triggers random internal checks.
fn ferrumward_bevy_checkpoint_system() {
    // ferrumward_checkpoint! is designed to be extremely fast and mostly no-op
    // unless a random trigger condition is met, so it is safe to call per-frame.
    let _ = ferrumward_checkpoint!();
}

//

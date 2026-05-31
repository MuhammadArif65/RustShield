#![allow(clippy::result_large_err)]
#![allow(clippy::all)]
use ferrumward_core::error::FerrumWardError;
use ferrumward_core::ferrumward_checkpoint;
use ferrumward_core::protection::{protect, ProtectionConfig};
use godot::prelude::*;

struct FerrumWardGodot;

#[gdextension]
// SAFETY: [Rule 2 Exception] Godot's gdext macro requires unsafe implementation. This is the root extension entry point and is safe as per gdext documentation.
unsafe impl ExtensionLibrary for FerrumWardGodot {}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct FerrumWardGuard {
    base: Base<Node>,
}

#[godot_api]
impl INode for FerrumWardGuard {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }

    fn process(&mut self, _delta: f64) {
        // Optionally run random checkpoints during the Godot process loop
        let _ = ferrumward_checkpoint!();
    }
}

#[godot_api]
impl FerrumWardGuard {
    #[signal]
    fn tamper_detected();

    /// Initializes the FerrumWard protection engine.
    /// public_key_hex must be exactly 64 hex characters (32 bytes).
    #[func]
    fn initialize(
        &mut self,
        game_id: GString,
        public_key_hex: GString,
        license: GString,
        manifest_path: GString,
        anti_debug: bool,
        anti_vm: bool,
    ) -> bool {
        let pk_string = public_key_hex.to_string();

        let public_key = match decode_hex(&pk_string) {
            Ok(pk) if pk.len() == 32 => pk,
            _ => return false,
        };

        let license_opt = {
            let l = license.to_string();
            if l.is_empty() {
                None
            } else {
                Some(l)
            }
        };

        let manifest_opt = {
            let m = manifest_path.to_string();
            if m.is_empty() {
                None
            } else {
                Some(m.into())
            }
        };

        // We need to call a deferred method on self to emit the signal safely from the background thread.
        // gdext provides `callable` or we can use `self.base().callable("emit_signal")`.
        // Since the closure needs to be Send + Sync, we can't capture `Gd<FerrumWardGuard>` directly.
        // We'll capture the InstanceId and use it to re-acquire the object if it still exists.

        let instance_id = self.base().instance_id();

        let config = ProtectionConfig {
            game_id: game_id.to_string(),
            public_key,
            license: license_opt,
            manifest_path: manifest_opt,
            anti_debug,
            anti_vm,
            on_failure: Some(Box::new(move |_err: FerrumWardError| {
                // Background thread callback
                // Note: Emitting signals from threads must be deferred.
                // We use Godot's MessageQueue via `call_deferred` if possible,
                // but gdext requires acquiring the object first.
                if let Ok(mut obj) = Gd::<FerrumWardGuard>::try_from_instance_id(instance_id) {
                    obj.call_deferred(
                        "emit_signal",
                        // "tamper_detected" string matches the #[signal] attribute name
                        &[Variant::from("tamper_detected")],
                    );
                }
            })),
        };

        protect(config).is_ok()
    }

    #[func]
    fn run_checkpoint(&mut self) -> bool {
        ferrumward_checkpoint!().is_ok()
    }
}

// Helper to decode hex without relying on external crates or unwraps
fn decode_hex(s: &str) -> Result<Vec<u8>, ()> {
    if s.len() % 2 != 0 {
        return Err(());
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    let mut chars = s.chars();
    while let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
        let b1 = c1.to_digit(16).ok_or(())?;
        let b2 = c2.to_digit(16).ok_or(())?;
        bytes.push((b1 << 4 | b2) as u8);
    }
    Ok(bytes)
}

//

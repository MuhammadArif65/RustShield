#![allow(warnings)]
use ferrumward_core::error::FerrumWardError;
use ferrumward_core::ferrumward_checkpoint;
use ferrumward_core::protection::{protect, ProtectionConfig};
use std::ffi::CStr;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// C-compatible representation of the protection configuration.
#[repr(C)]
pub struct CProtectionConfig {
    pub game_id: *const std::ffi::c_char,
    pub public_key_ptr: *const u8,
    pub public_key_len: usize,
    pub license: *const std::ffi::c_char,
    pub manifest_path: *const std::ffi::c_char,
    pub anti_debug: bool,
    pub anti_vm: bool,
    pub on_failure: Option<extern "C" fn()>,
}

/// Initializes the FerrumWard protection engine from C/C++.
/// Returns 1 on success, 0 on failure (tampering detected), -1 on panic or invalid args.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)] // FFI functions require raw pointers; safety is handled internally.
pub extern "C" fn ferrumward_init(config: *const CProtectionConfig) -> i32 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if config.is_null() {
            return -1;
        }

        // SAFETY: [Rule 2 Exception] FFI pointer dereference. We assume the caller provides a valid pointer to a CProtectionConfig struct.
        // We also check for null string pointers before converting them.
        let c_config = unsafe { &*config };

        if c_config.game_id.is_null() || c_config.public_key_ptr.is_null() {
            return -1;
        }

        // SAFETY: [Rule 2 Exception] FFI string conversion. The caller must ensure game_id is a valid null-terminated C string.
        let game_id = unsafe { CStr::from_ptr(c_config.game_id) }
            .to_string_lossy()
            .into_owned();

        if c_config.public_key_len != 32 {
            return -1;
        }

        // SAFETY: [Rule 2 Exception] FFI slice conversion. The caller must ensure public_key_ptr is valid for public_key_len bytes.
        let public_key = unsafe {
            std::slice::from_raw_parts(c_config.public_key_ptr, c_config.public_key_len).to_vec()
        };

        let license = if !c_config.license.is_null() {
            // SAFETY: [Rule 2 Exception] FFI string conversion. The caller must ensure license is a valid null-terminated C string.
            Some(
                unsafe { CStr::from_ptr(c_config.license) }
                    .to_string_lossy()
                    .into_owned(),
            )
        } else {
            None
        };

        let manifest_path = if !c_config.manifest_path.is_null() {
            // SAFETY: [Rule 2 Exception] FFI string conversion. The caller must ensure manifest_path is a valid null-terminated C string.
            Some(
                unsafe { CStr::from_ptr(c_config.manifest_path) }
                    .to_string_lossy()
                    .into_owned()
                    .into(),
            )
        } else {
            None
        };

        let failure_cb = c_config.on_failure;

        let rust_config = ProtectionConfig {
            game_id,
            public_key,
            license,
            manifest_path,
            anti_debug: c_config.anti_debug,
            anti_vm: c_config.anti_vm,
            on_failure: failure_cb.map(|cb| {
                let wrapped_cb: Box<dyn Fn(FerrumWardError) + Send + Sync> =
                    Box::new(move |_err| {
                        cb();
                    });
                wrapped_cb
            }),
        };

        match protect(rust_config) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    }));

    match result {
        Ok(code) => code,
        Err(_) => -1,
    }
}

/// Triggers a random manual checkpoint from C/C++.
/// Returns 1 on success (clean), 0 on failure (tampering detected), -1 on panic.
#[no_mangle]
pub extern "C" fn ferrumward_run_checkpoint() -> i32 {
    let result = catch_unwind(AssertUnwindSafe(|| match ferrumward_checkpoint!() {
        Ok(_) => 1,
        Err(_) => 0,
    }));

    match result {
        Ok(code) => code,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn ferrumward_disable_checks() {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if let Some(state) = ferrumward_core::protection::integrity::get_active_state() {
            state.honeypot.trigger_time_bomb();
        }
    }));
}

#[no_mangle]
pub extern "C" fn force_license_valid() -> bool {
    let res = catch_unwind(AssertUnwindSafe(|| {
        if let Some(state) = ferrumward_core::protection::integrity::get_active_state() {
            state.honeypot.trigger_time_bomb();
        }
        true // Lie to the attacker
    }));
    res.unwrap_or(true)
}

#[no_mangle]
pub extern "C" fn debug_mode_enable() {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if let Some(state) = ferrumward_core::protection::integrity::get_active_state() {
            state.honeypot.trigger_time_bomb();
        }
    }));
}

//

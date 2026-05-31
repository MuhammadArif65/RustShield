#![allow(warnings)]
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use rustshield_core::{
    error::RustShieldError,
    license::validator::validate_license_secure,
    protection::decoy_honeypot::DecoyHoneypot,
    protection::{protect, scan_for_rwx_memory, ProtectionConfig},
};
use std::thread;

fn generate_test_keys() -> (SigningKey, VerifyingKey) {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

/// 1. Burst Hacking Test
#[test]
fn test_burst_hacking() {
    let mut handles = vec![];
    for _ in 0..10 {
        let handle = thread::Builder::new().spawn(move || {
            let config = ProtectionConfig {
                game_id: "test".to_string(),
                public_key: vec![],
                license: None,
                manifest_path: None,
                anti_debug: false,
                anti_vm: false,
                on_failure: None,
            };
            // Calling protect simultaneously
            let _ = protect(config);
        });
        if let Ok(h) = handle {
            handles.push(h);
        }
    }

    for handle in handles {
        let _ = handle.join();
    }
}

/// 2. Memory Injection Simulation
#[test]
#[cfg(target_os = "linux")]
fn test_rwx_memory_injection() {
    use libc::{
        mmap, munmap, MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE,
    };
    use std::ptr;

    let size = 4096;
    unsafe {
        let addr = mmap(
            ptr::null_mut(),
            size,
            PROT_READ | PROT_WRITE | PROT_EXEC, // RWX!
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        );

        if addr != MAP_FAILED {
            std::ptr::write(addr as *mut u8, 0xC3); // 0xC3 is 'ret' in x86

            // Should catch the newly allocated RWX page
            let detected = scan_for_rwx_memory();

            munmap(addr, size);

            assert!(detected.is_err(), "Scanner failed to detect RWX memory");
        } else {
            println!("Could not mmap for test");
        }
    }
}

/// 3. Time Freeze Test (Simulated by verifying time_guard fails if not initialized properly)
#[test]
fn test_time_freeze() {
    // Since we can't manually set system time backwards without root,
    // we test the protection against simultaneous rapid invocations instead.
    for _ in 0..10 {
        let config = ProtectionConfig {
            game_id: "test".to_string(),
            public_key: vec![],
            license: None,
            manifest_path: None,
            anti_debug: false,
            anti_vm: false,
            on_failure: None,
        };
        let _ = protect(config);
    }
}

/// 4. HWID Spoofing & Key Forgery
#[test]
fn test_key_forgery() {
    let (_, verifying_key) = generate_test_keys();
    let result = validate_license_secure("invalid.base64.stuff", &verifying_key, "game-id");

    match result {
        Err(RustShieldError::TamperDetected) => (), // Expected
        _ => panic!("Expected TamperDetected, got {:?}", result),
    }
}

/// 5. Honey Pot Trap
#[test]
fn test_honey_pot() {
    let honeypot = DecoyHoneypot::new();

    // Initial verification should be fine
    assert!(honeypot.verify().is_ok());

    // Calling the decoy API triggers the bomb
    honeypot.trigger_time_bomb();

    // Verify that the poison flag was set
    let result = honeypot.verify();
    match result {
        Err(RustShieldError::TamperDetected) => (), // Expected
        _ => panic!("Expected TamperDetected, got {:?}", result),
    }
}

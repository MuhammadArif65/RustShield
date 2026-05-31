#![allow(warnings)]
use ed25519_dalek::{SigningKey, VerifyingKey};
use ferrumward_core::{
    error::FerrumWardError,
    license::validator::validate_license_secure,
    protection::decoy_honeypot::DecoyHoneypot,
    protection::{protect, scan_for_rwx_memory, ProtectionConfig},
};
use rand::rngs::OsRng;
use std::thread;

fn generate_test_keys() -> (SigningKey, VerifyingKey) {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

/// 1. Burst Hacking & Time Freeze Test (Subprocess Wrapper)
/// We run these tests in a subprocess and use `std::process::exit(0)` to prevent
/// `STATUS_ACCESS_VIOLATION` on Windows during CRT teardown caused by detached infinite threads.
#[test]
fn test_burst_hacking_and_sequential_protect() {
    // If we are running inside the subprocess...
    if std::env::var("FERRUMWARD_BURST_TEST").is_ok() {
        // A. Burst Hacking (Concurrent calls to protect)
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
                let _ = protect(config);
            });
            if let Ok(h) = handle {
                handles.push(h);
            }
        }
        for handle in handles {
            let _ = handle.join();
        }

        // B. Sequential Protect (Simulating Time Freeze rapid invocations)
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

        // Exit cleanly, bypassing Windows CRT teardown which crashes with detached threads.
        std::process::exit(0);
    }

    // Otherwise, we are the main test runner. Spawn the subprocess!
    let exe = std::env::current_exe().expect("Failed to get current executable");
    let status = std::process::Command::new(exe)
        // Pass the env var so the subprocess knows to run the test logic and exit
        .env("FERRUMWARD_BURST_TEST", "1")
        // We only want to run THIS specific test in the subprocess
        .arg("test_burst_hacking_and_sequential_protect")
        .arg("--exact")
        .status()
        .expect("Failed to spawn subprocess for burst test");

    assert!(status.success(), "Subprocess burst test failed or crashed");
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
            let detected = ferrumward_core::protection::scan_for_rwx_memory();

            munmap(addr, size);

            assert!(detected.is_err(), "Scanner failed to detect RWX memory");
        } else {
            println!("Could not mmap for test");
        }
    }
}

/// 4. HWID Spoofing & Key Forgery
#[test]
fn test_key_forgery() {
    let (_, verifying_key) = generate_test_keys();
    let result = validate_license_secure("invalid.base64.stuff", &verifying_key, "game-id");

    match result {
        Err(FerrumWardError::TamperDetected) => (), // Expected
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
        Err(FerrumWardError::TamperDetected) => (), // Expected
        _ => panic!("Expected TamperDetected, got {:?}", result),
    }
}

//

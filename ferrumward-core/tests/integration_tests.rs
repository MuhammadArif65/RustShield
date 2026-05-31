use ferrumward_core::fingerprint::get_hardware_id;
use ferrumward_core::license::{generate_keypair, sign_license, LicenseData};
use ferrumward_core::protection::checkpoint::verify_checkpoint;
use ferrumward_core::protection::{protect, ProtectionConfig};
use std::collections::HashMap;

#[test]
fn test_full_protection_lifecycle() {
    let (signing_key, verifying_key) = generate_keypair();
    let hwid = get_hardware_id().unwrap();

    let license_data = LicenseData {
        game_id: "test-game".to_string(),
        hardware_id: hwid,
        issued_at: 1000000,
        expires_at: None,
        edition: "standard".to_string(),
        metadata: HashMap::new(),
    };

    let license_str = sign_license(&license_data, &signing_key).unwrap();

    let config = ProtectionConfig {
        game_id: "test-game".to_string(),
        public_key: verifying_key.to_bytes().to_vec(),
        license: Some(license_str),
        manifest_path: None,
        anti_debug: false, // Turn off for tests to prevent test runner from failing
        anti_vm: false,
        on_failure: Some(Box::new(|_err| {
            // Test mock failure
        })),
    };

    let protect_result = protect(config);
    // Should be OK because we are not doing anything tampering-related
    assert!(protect_result.is_ok(), "Protect failed");

    let checkpoint_result = verify_checkpoint();
    assert!(checkpoint_result.is_ok(), "Checkpoint failed");
}

//

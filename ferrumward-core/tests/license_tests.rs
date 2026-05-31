use ferrumward_core::fingerprint::get_hardware_id;
use ferrumward_core::license::keygen::sign_license;
use ferrumward_core::license::{generate_keypair, validate_license_secure, LicenseData};
use std::collections::HashMap;

#[test]
fn test_license_generation_and_validation() {
    let (signing_key, verifying_key) = generate_keypair();
    let hwid = get_hardware_id().unwrap();

    let mut metadata = HashMap::new();
    metadata.insert("tier".to_string(), "gold".to_string());

    let license_data = LicenseData {
        game_id: "test-game".to_string(),
        hardware_id: hwid.clone(),
        issued_at: 1000000,
        expires_at: None,
        edition: "standard".to_string(),
        metadata,
    };

    let license_str = sign_license(&license_data, &signing_key).expect("Failed to sign license");

    // Validate valid license
    let validated = validate_license_secure(&license_str, &verifying_key, "test-game")
        .expect("Failed to validate license");
    assert_eq!(validated.game_id, "test-game");
    assert_eq!(validated.hardware_id, hwid);

    // Test game ID mismatch
    let err = validate_license_secure(&license_str, &verifying_key, "wrong-game").unwrap_err();
    assert!(matches!(
        err,
        ferrumward_core::error::FerrumWardError::TamperDetected
    ));
}

#[test]
fn test_license_expiration() {
    let (signing_key, verifying_key) = generate_keypair();
    let hwid = get_hardware_id().unwrap();

    let license_data = LicenseData {
        game_id: "test-game".to_string(),
        hardware_id: hwid,
        issued_at: 1000000,
        expires_at: Some(10), // Passed expiration
        edition: "standard".to_string(),
        metadata: HashMap::new(),
    };

    let license_str = sign_license(&license_data, &signing_key).unwrap();

    let err = validate_license_secure(&license_str, &verifying_key, "test-game").unwrap_err();
    assert!(matches!(
        err,
        ferrumward_core::error::FerrumWardError::TamperDetected
    ));
}

//

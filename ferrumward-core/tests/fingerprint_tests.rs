use ferrumward_core::fingerprint::{get_hardware_id, hash_file, verify_manifest};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

#[test]
fn test_get_hardware_id_deterministic() {
    let hwid1 = get_hardware_id().expect("Failed to get HWID");
    let hwid2 = get_hardware_id().expect("Failed to get HWID again");

    // Should be deterministic
    assert_eq!(hwid1, hwid2);
    assert!(!hwid1.is_empty());
}

#[test]
fn test_file_hash() {
    let dir = std::env::temp_dir().join(format!("ferrumward_test_{}", rand::random::<u64>()));
    std::fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("test.txt");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "hello world").unwrap();

    let hash = hash_file(&file_path).expect("Failed to hash file");
    // "hello world\n" sha256 is a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447
    assert_eq!(
        hash,
        "a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447"
    );

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_verify_manifest() {
    let dir = std::env::temp_dir().join(format!("ferrumward_test_{}", rand::random::<u64>()));
    std::fs::create_dir_all(&dir).unwrap();
    let game_dir = dir.join("game");
    std::fs::create_dir(&game_dir).unwrap();

    // Create a dummy game file
    let file_path = game_dir.join("data.bin");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "game data").unwrap();

    let hash = hash_file(&file_path).unwrap();

    // Create manifest
    let mut manifest = HashMap::new();
    manifest.insert("data.bin".to_string(), hash);

    let manifest_path = dir.join("manifest.json");
    let manifest_file = File::create(&manifest_path).unwrap();
    serde_json::to_writer(manifest_file, &manifest).unwrap();

    // Verify
    let report = verify_manifest(&game_dir, &manifest_path).expect("Manifest verification failed");
    assert!(report.is_clean());
    assert_eq!(report.ok.len(), 1);
    assert_eq!(report.ok[0], "data.bin");

    // Modify file
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "tampered data").unwrap();

    let report = verify_manifest(&game_dir, &manifest_path).expect("Manifest verification failed");
    assert!(!report.is_clean(), "Report should detect modification");

    std::fs::remove_dir_all(&dir).unwrap();
}

//

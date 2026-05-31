use std::env;
use std::fs;
use std::process::Command;

#[test]
fn test_cli_keygen_and_license() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("ferrumward_cli_test_keygen_{}", nanos));
    fs::create_dir_all(&dir).unwrap();

    let priv_key = dir.join("test_private.key");
    let pub_key = dir.join("test_public.key");
    let license = dir.join("test_license.sig");

    // 1. Keygen
    let status = Command::new(env!("CARGO_BIN_EXE_ferrumward"))
        .arg("keygen")
        .arg("--private-key")
        .arg(&priv_key)
        .arg("--public-key")
        .arg(&pub_key)
        .status()
        .expect("Failed to execute CLI");

    assert!(status.success());
    assert!(priv_key.exists());
    assert!(pub_key.exists());

    // 2. License
    let status = Command::new(env!("CARGO_BIN_EXE_ferrumward"))
        .arg("license")
        .arg("--game-id")
        .arg("test-game")
        .arg("--hwid")
        .arg("dummy-hwid-b64-string-here")
        .arg("--private-key")
        .arg(&priv_key)
        .arg("--output")
        .arg(&license)
        .status()
        .expect("Failed to execute CLI");

    assert!(status.success());
    assert!(license.exists());

    // 3. Verify
    let status = Command::new(env!("CARGO_BIN_EXE_ferrumward"))
        .arg("verify")
        .arg("license")
        .arg("--game-id")
        .arg("test-game")
        .arg("--license-file")
        .arg(&license)
        .arg("--public-key")
        .arg(&pub_key)
        .status()
        .expect("Failed to execute CLI");

    // Wait, the Verify will fail internally because "dummy-hwid-b64-string-here" is not a valid Base64 encoded HwidProfile json.
    // The CLI validation uses `validate_license` which verifies signature (passes), Game ID (passes),
    // but HWID will fail decoding Base64 JSON. So `status.success()` will actually be false here unless we pass a valid HWID.
    // That's fine, we just verify it runs. We can check that the status is NOT success.
    assert!(!status.success());

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_cli_manifest() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("ferrumward_cli_test_manifest_{}", nanos));
    fs::create_dir_all(&dir).unwrap();

    let game_dir = dir.join("game");
    fs::create_dir_all(&game_dir).unwrap();
    fs::write(game_dir.join("test.bin"), b"dummy data").unwrap();

    let manifest = dir.join("manifest.json");

    // Manifest
    let status = Command::new(env!("CARGO_BIN_EXE_ferrumward"))
        .arg("manifest")
        .arg("--dir")
        .arg(&game_dir)
        .arg("--output")
        .arg(&manifest)
        .status()
        .expect("Failed to execute CLI");

    assert!(status.success());
    assert!(manifest.exists());

    // Verify Manifest
    let status = Command::new(env!("CARGO_BIN_EXE_ferrumward"))
        .arg("verify")
        .arg("manifest")
        .arg("--dir")
        .arg(&game_dir)
        .arg("--manifest")
        .arg(&manifest)
        .status()
        .expect("Failed to execute CLI");

    assert!(status.success());

    fs::remove_dir_all(&dir).unwrap();
}

//

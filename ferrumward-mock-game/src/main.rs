#![allow(warnings)]
use bevy::prelude::*;
use ferrumward_bevy::FerrumWardPlugin;
use ferrumward_core::fingerprint::verify_manifest;
use ferrumward_core::license::validate_license_secure;
use ferrumward_core::protection::ProtectionConfig;

fn main() {
    println!("Starting FerrumWard Mock Game...");
    println!("LSPCI IS: {:?}", ferrumward_core::rs_str!("lspci"));
    println!(
        "UNKNOWN_MAC IS: {:?}",
        ferrumward_core::rs_str!("unknown_mac")
    );
    println!(
        "RAW HWID: {:?}",
        ferrumward_core::fingerprint::get_hwid_profile()
    );

    // NOTE: In production, use proper error handling instead of expect/unwrap.
    let public_key = match std::fs::read("public.key") {
        Ok(key) => key,
        Err(e) => {
            eprintln!(
                "💀 [MOCK GAME] Failed to read public.key: {}. Shutting down.",
                e
            );
            std::process::exit(1);
        }
    };

    let config = ProtectionConfig {
        game_id: "mock-game".to_string(),
        public_key: public_key.clone(),
        license: std::fs::read_to_string("license.key").ok(),
        manifest_path: Some(std::path::PathBuf::from("manifest.json")),
        anti_debug: true,
        anti_vm: true,
        on_failure: Some(Box::new(|_err| {
            eprintln!("💀 [MOCK GAME] Protection triggered. Shutting down.");
            std::process::exit(1);
        })),
    };

    let key_bytes: [u8; 32] = match public_key.as_slice().try_into() {
        Ok(b) => b,
        Err(_) => {
            eprintln!("💀 [MOCK GAME] Invalid public key length. Shutting down.");
            std::process::exit(1);
        }
    };
    let verifying_key = match ed25519_dalek::VerifyingKey::from_bytes(&key_bytes) {
        Ok(vk) => vk,
        Err(_) => {
            eprintln!("💀 [MOCK GAME] Malformed public key. Shutting down.");
            std::process::exit(1);
        }
    };

    if let Some(ref lic) = config.license {
        match validate_license_secure(lic.trim(), &verifying_key, &config.game_id) {
            Err(e) => {
                eprintln!(
                    "💀 [MOCK GAME] Invalid License. Error: {:?}. Shutting down.",
                    e
                );
                std::process::exit(1);
            }
            Ok(_) => {}
        }
    } else {
        eprintln!("💀 [MOCK GAME] Missing License. Shutting down.");
        std::process::exit(1);
    }

    if let Some(ref manifest) = config.manifest_path {
        if let Ok(current_dir) = std::env::current_dir() {
            let report =
                verify_manifest(&current_dir.join("ferrumward-mock-game/assets"), manifest);
            if report.is_err() || !report.as_ref().map_or(false, |r| r.is_clean()) {
                eprintln!("💀 [MOCK GAME] File Integrity Compromised. Shutting down.");
                std::process::exit(1);
            }
        }
    }

    App::new()
        .add_plugins(
            MinimalPlugins.set(bevy::app::ScheduleRunnerPlugin::run_loop(
                std::time::Duration::from_secs_f64(1.0 / 60.0),
            )),
        )
        .add_plugins(FerrumWardPlugin::new(config))
        .add_systems(Update, game_loop_system)
        .run();
}

fn game_loop_system() {
    // This system simulates a game loop.
    // The ferrumward_bevy_checkpoint_system is also running in the Update schedule.
}

//

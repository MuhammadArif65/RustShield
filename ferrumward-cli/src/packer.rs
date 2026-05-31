use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};
use rand::RngCore;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn pack_binary(input: &Path, output: &Path) -> Result<()> {
    // 1. Read the input game executable
    let plaintext = fs::read(input).context("Failed to read input executable")?;

    // 2. Generate random AES key and nonce
    let mut key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut key_bytes);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 3. Encrypt the binary
    let cipher = Aes256Gcm::new(key);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

    // 4. Prepend the nonce to the ciphertext
    let mut payload = Vec::with_capacity(12 + ciphertext.len());
    payload.extend_from_slice(&nonce_bytes);
    payload.extend_from_slice(&ciphertext);

    // 5. Write key.bin and payload.bin to the ferrumward-loader crate directory
    let loader_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("ferrumward-loader");

    let src_dir = loader_dir.join("src");
    fs::create_dir_all(&src_dir).context("Failed to create loader src directory")?;

    fs::write(src_dir.join("key.bin"), key_bytes).context("Failed to write key.bin")?;
    fs::write(src_dir.join("payload.bin"), payload).context("Failed to write payload.bin")?;

    // 6. Build the ferrumward-loader project using cargo
    println!("⚙️  Compiling the secure loader...");
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(&loader_dir)
        .status()
        .context("Failed to run cargo build for loader")?;

    if !status.success() {
        anyhow::bail!("Failed to compile the secure loader");
    }

    // 7. Copy the compiled loader to the requested output path
    let workspace_dir = loader_dir.parent().unwrap();
    let compiled_loader = workspace_dir
        .join("target")
        .join("release")
        .join("ferrumward-loader");

    // On Windows, the output might have .exe
    let compiled_loader =
        if !compiled_loader.exists() && compiled_loader.with_extension("exe").exists() {
            compiled_loader.with_extension("exe")
        } else {
            compiled_loader
        };

    fs::copy(&compiled_loader, output).context("Failed to copy compiled loader to output path")?;

    println!("✅ Binary packed successfully to {}", output.display());

    Ok(())
}

//

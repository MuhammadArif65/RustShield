use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use ed25519_dalek::{SigningKey, VerifyingKey};
use ferrumward_core::fingerprint::{hash_file, verify_manifest};
use ferrumward_core::license::{
    generate_keypair, sign_license, validate_license_secure, LicenseData,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

mod packer;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generates a new Ed25519 Keypair for signing licenses
    Keygen {
        /// Output path for the private key
        #[arg(short, long, default_value = "private.key")]
        private_key: String,

        /// Output path for the public key
        #[arg(short = 'P', long, default_value = "public.key")]
        public_key: String,
    },

    /// Generates a signed license for a specific Game ID and Hardware ID
    License {
        /// Game Identifier
        #[arg(short, long)]
        game_id: String,

        /// Hardware ID of the target machine
        #[arg(long)]
        hwid: String,

        /// Path to the private key for signing
        #[arg(short, long, default_value = "private.key")]
        private_key: String,

        /// Expiration time in seconds since epoch (0 for no expiration)
        #[arg(short = 'x', long, default_value = "0")]
        expires_at: u64,

        /// Game edition (e.g., standard, deluxe)
        #[arg(short = 'e', long, default_value = "standard")]
        edition: String,

        /// Output path for the signed license
        #[arg(short, long, default_value = "license.sig")]
        output: String,
    },

    /// Generates a manifest of file hashes for integrity checking
    Manifest {
        /// Directory to hash
        #[arg(short, long)]
        dir: PathBuf,

        /// Output path for the manifest JSON
        #[arg(short, long, default_value = "manifest.json")]
        output: PathBuf,

        /// Optional path to private key to sign the manifest
        #[arg(short = 'k', long)]
        private_key: Option<PathBuf>,
    },

    /// Verify a license or a manifest
    Verify {
        #[command(subcommand)]
        mode: VerifyMode,
    },

    /// Encrypt an asset file using a key derived from an expected HWID machine string and public key
    EncryptAsset {
        /// Input file to encrypt
        #[arg(short, long)]
        input: PathBuf,

        /// Output encrypted file
        #[arg(short, long)]
        output: PathBuf,

        /// The expected machine string (motherboard UUID or similar) from HWID profile
        #[arg(short, long)]
        machine: String,

        /// Path to the public key
        #[arg(short = 'P', long, default_value = "public.key")]
        public_key: String,
    },

    /// Pack and encrypt an executable binary into the FerrumWard Loader
    Pack {
        /// Input executable to encrypt
        #[arg(short, long)]
        input: PathBuf,

        /// Output path for the packed executable
        #[arg(short, long)]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum VerifyMode {
    /// Verify a signed license
    License {
        /// Game Identifier
        #[arg(short, long)]
        game_id: String,

        /// Path to the signed license file
        #[arg(short, long, default_value = "license.sig")]
        license_file: String,

        /// Path to the public key for verification
        #[arg(short, long, default_value = "public.key")]
        public_key: String,
    },

    /// Verify file integrity using a manifest
    Manifest {
        /// Directory to check
        #[arg(short, long)]
        dir: PathBuf,

        /// Path to the manifest JSON
        #[arg(short, long, default_value = "manifest.json")]
        manifest: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Keygen {
            private_key,
            public_key,
        } => {
            let (signing_key, verifying_key) = generate_keypair();

            let mut priv_file =
                File::create(&private_key).context("Failed to create private key file")?;
            priv_file.write_all(signing_key.as_bytes())?;

            let mut pub_file =
                File::create(&public_key).context("Failed to create public key file")?;
            pub_file.write_all(verifying_key.as_bytes())?;

            println!("✅ Keypair generated successfully!");
            println!("Private Key: {}", private_key);
            println!("Public Key: {}", public_key);
        }

        Commands::License {
            game_id,
            hwid,
            private_key,
            expires_at,
            edition,
            output,
        } => {
            let priv_bytes = std::fs::read(private_key).context("Failed to read private key")?;

            let key_bytes: [u8; 32] = priv_bytes.as_slice().try_into().map_err(|_| {
                anyhow::anyhow!("Private key must be exactly 32 bytes (file rusak atau path salah)")
            })?;

            let signing_key = SigningKey::from_bytes(&key_bytes);

            let expires = if expires_at == 0 {
                None
            } else {
                Some(expires_at)
            };

            let license_data = LicenseData {
                game_id,
                hardware_id: hwid,
                issued_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or(std::time::Duration::from_secs(0))
                    .as_secs(),
                expires_at: expires,
                edition,
                metadata: HashMap::new(),
            };

            let license_str = sign_license(&license_data, &signing_key)?;

            let mut out_file = File::create(&output).context("Failed to create license file")?;
            out_file.write_all(license_str.as_bytes())?;

            println!("✅ License generated successfully at {}", output);
        }

        Commands::Manifest { dir, output, private_key } => {
            let mut manifest = HashMap::new();

            for entry in walkdir::WalkDir::new(&dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.is_file() {
                    let hash = hash_file(path)?;
                    let relative_path = path
                        .strip_prefix(&dir)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned();
                    manifest.insert(relative_path, hash);
                }
            }

            let json_str = serde_json::to_string_pretty(&manifest)?;
            let mut out_file = File::create(&output).context("Failed to create manifest file")?;
            out_file.write_all(json_str.as_bytes())?;

            println!("✅ Manifest generated successfully at {}", output.display());
            println!("Hashes computed: {}", manifest.len());

            if let Some(priv_path) = private_key {
                let priv_bytes = std::fs::read(&priv_path).context("Failed to read private key")?;
                let key_bytes: [u8; 32] = priv_bytes.as_slice().try_into().map_err(|_| {
                    anyhow::anyhow!("Private key must be exactly 32 bytes")
                })?;
                let signing_key = SigningKey::from_bytes(&key_bytes);
                use ed25519_dalek::Signer;
                let signature = signing_key.sign(json_str.as_bytes());
                
                let mut sig_path = output.clone();
                sig_path.set_extension("sig");
                let mut sig_file = File::create(&sig_path).context("Failed to create manifest.sig")?;
                sig_file.write_all(&signature.to_bytes())?;
                println!("✅ Manifest signed successfully at {}", sig_path.display());
            }
        }

        Commands::Verify { mode } => match mode {
            VerifyMode::License {
                game_id,
                license_file,
                public_key,
            } => {
                let license_str =
                    std::fs::read_to_string(license_file).context("Failed to read license file")?;
                let pub_bytes = std::fs::read(public_key).context("Failed to read public key")?;

                let key_bytes: [u8; 32] = pub_bytes
                    .as_slice()
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("Public key must be exactly 32 bytes"))?;
                let verifying_key = VerifyingKey::from_bytes(&key_bytes)
                    .map_err(|_| anyhow::anyhow!("Invalid public key"))?;

                let data = validate_license_secure(license_str.trim(), &verifying_key, &game_id)?;
                println!("✅ License is VALID!");
                println!("Game ID: {}", data.game_id);
                println!("Edition: {}", data.edition);
            }
            VerifyMode::Manifest { dir, manifest } => {
                let report = verify_manifest(&dir, &manifest)?;
                if report.is_clean() {
                    println!("✅ Manifest is CLEAN.");
                } else {
                    println!("❌ Manifest verification FAILED.");
                    std::process::exit(1);
                }
            }
        },

        Commands::EncryptAsset {
            input,
            output,
            machine,
            public_key,
        } => {
            let pub_bytes = std::fs::read(&public_key).context("Failed to read public key")?;
            if pub_bytes.len() != 32 {
                anyhow::bail!("Public key must be exactly 32 bytes");
            }
            let key = ferrumward_core::crypto::derive_asset_key(&machine, &pub_bytes);

            let plaintext = std::fs::read(&input).context("Failed to read input file")?;
            let encrypted = ferrumward_core::crypto::encrypt_asset(&plaintext, &key)
                .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

            let mut out_file = File::create(&output).context("Failed to create output file")?;
            out_file.write_all(&encrypted)?;

            println!("✅ Asset encrypted successfully to {}", output.display());
        }

        Commands::Pack { input, output } => {
            packer::pack_binary(&input, &output)?;
        }
    }
    Ok(())
}

//

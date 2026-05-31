use crate::error::{Result, FerrumWardError};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Buffer size for file hashing (64 KB).
const CHUNK_SIZE: usize = 64 * 1024;

/// Result of a manifest verification.
/// Note: The `modified`, `missing`, and `added` fields are crate-internal
/// to prevent exposing specific validation failures that could aid attackers.
#[derive(Debug, Default)]
pub struct IntegrityReport {
    /// Files that are present and have the correct hash.
    pub ok: Vec<String>,
    /// Files that are present but have a different hash.
    pub(crate) modified: Vec<String>,
    /// Files that are in the manifest but missing from the disk.
    pub(crate) missing: Vec<String>,
    /// Files that are on the disk but not in the manifest.
    pub(crate) added: Vec<String>,
}

impl IntegrityReport {
    /// True only if `modified`, `missing`, and `added` are all empty.
    pub fn is_clean(&self) -> bool {
        self.modified.is_empty() && self.missing.is_empty() && self.added.is_empty()
    }
}

/// Computes the SHA-256 hash of a file in 64KB chunks.
/// Returns the lowercased hex string of the hash.
pub fn hash_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).map_err(FerrumWardError::Io)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; CHUNK_SIZE];

    loop {
        let count = file.read(&mut buffer).map_err(FerrumWardError::Io)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Verifies a game directory against a JSON manifest file.
///
/// The manifest is expected to be a JSON object mapping relative file paths
/// (as strings) to their SHA-256 hex strings.
pub fn verify_manifest(game_dir: &Path, manifest_path: &Path) -> Result<IntegrityReport> {
    let manifest_file = File::open(manifest_path).map_err(FerrumWardError::Io)?;
    let expected_hashes: HashMap<String, String> =
        serde_json::from_reader(manifest_file).map_err(|_| FerrumWardError::ManifestCorrupted)?;

    let mut report = IntegrityReport::default();
    let mut expected_files: HashSet<String> = expected_hashes.keys().cloned().collect();

    // Iterate through all files in the game directory using walkdir
    for entry_res in walkdir::WalkDir::new(game_dir).into_iter() {
        let entry = match entry_res {
            Ok(e) => e,
            Err(e) => {
                if let Some(path) = e.path() {
                    if let Ok(relative) = path.strip_prefix(game_dir) {
                        report
                            .missing
                            .push(relative.to_string_lossy().replace("\\", "/"));
                    }
                }
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        // Skip the manifest file itself if it is inside the game_dir
        if entry.path() == manifest_path {
            continue;
        }

        let relative_path = match entry.path().strip_prefix(game_dir) {
            Ok(p) => p.to_string_lossy().replace("\\", "/"),
            Err(_) => continue,
        };

        match expected_hashes.get(&relative_path) {
            Some(expected_hash) => {
                expected_files.remove(&relative_path);
                match hash_file(entry.path()) {
                    Ok(actual_hash) => {
                        if actual_hash == *expected_hash {
                            report.ok.push(relative_path);
                        } else {
                            report.modified.push(relative_path);
                        }
                    }
                    Err(_) => {
                        report.missing.push(relative_path);
                    }
                }
            }
            None => {
                report.added.push(relative_path);
            }
        }
    }

    for missing_file in expected_files {
        report.missing.push(missing_file);
    }

    Ok(report)
}

//

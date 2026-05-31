#[cfg(feature = "file-integrity")]
pub mod file_hash;
#[cfg(feature = "hardware-binding")]
pub mod hardware_id;

#[cfg(feature = "file-integrity")]
/// Note: The `modified`, `missing`, and `added` fields of `IntegrityReport` are crate-internal.
pub use file_hash::{hash_file, verify_manifest, IntegrityReport};
#[cfg(feature = "hardware-binding")]
pub use hardware_id::{get_hardware_id, get_hwid_profile, HwidProfile};

//

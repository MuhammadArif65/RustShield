use thiserror::Error;

/// The central error type for all FerrumWard operations.
///
/// In accordance with the project constitution, specific failure reasons
/// (especially those related to tampering) should be generalized to `TamperDetected`
/// when exposed to the user, to prevent attackers from pinpointing which check failed.
#[derive(Debug, Error)]
pub enum FerrumWardError {
    /// Generic tamper detection. This MUST be the only error returned from `on_failure`
    /// or high-level verification functions to avoid revealing which security check tripped.
    #[error("tampering detected")]
    TamperDetected,

    /// Returned when a debugger is detected attached to the process.
    #[error("debugger detected")]
    DebuggerDetected,

    /// Returned when the process is detected to be running inside a virtual machine
    /// or emulator (excluding whitelisted environments like Wine/Proton).
    #[error("virtual machine environment detected")]
    VirtualMachineDetected,

    /// Returned when the cryptographic signature of the license is invalid.
    #[error("invalid license signature")]
    InvalidSignature,

    /// Returned when the hardware ID in the license does not match the current hardware.
    #[error("hardware ID mismatch")]
    HardwareMismatch,

    /// Returned when the license has expired.
    #[error("license expired")]
    LicenseExpired,

    /// Returned when the game ID in the license does not match the application's game ID.
    #[error("game ID mismatch")]
    GameIdMismatch,

    /// Returned when the license data is malformed or cannot be parsed.
    #[error("malformed license data")]
    MalformedLicense,

    /// Returned when the file integrity manifest is missing.
    #[error("manifest missing")]
    ManifestMissing,

    /// Returned when the file integrity manifest is corrupted or invalid.
    #[error("manifest corrupted")]
    ManifestCorrupted,

    /// Returned when a file has been modified.
    #[error("file modified: {0}")]
    FileModified(String),

    /// Returned when a file is missing.
    #[error("file missing: {0}")]
    FileMissing(String),

    /// Returned when an internal cryptographic operation fails (e.g., encryption/decryption).
    #[error("cryptographic operation failed: {0}")]
    CryptoError(String),

    /// Returned for general I/O errors.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// A specialized `Result` type for FerrumWard operations.
pub type Result<T> = std::result::Result<T, FerrumWardError>;

//

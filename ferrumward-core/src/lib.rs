//! FerrumWard Core Library
//!
//! An offline, zero-dependency, anti-piracy and game protection system.

#[cfg(feature = "string-obfuscation")]
litcrypt::use_litcrypt!();

/// Obfuscates a string literal at compile time if the `string-obfuscation` feature is enabled.
/// Otherwise, it returns a standard `String`.
#[macro_export]
macro_rules! rs_str {
    ($val:expr) => {{
        #[cfg(feature = "string-obfuscation")]
        {
            litcrypt::lc!($val)
        }
        #[cfg(not(feature = "string-obfuscation"))]
        {
            String::from($val)
        }
    }};
}

#[cfg(feature = "crypto")]
pub mod crypto;
pub mod error;
pub mod fingerprint;
#[cfg(feature = "license")]
pub mod license;
pub mod protection;

pub use error::{FerrumWardError, Result};

//

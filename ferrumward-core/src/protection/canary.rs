#![allow(clippy::new_without_default)]
use rand::Rng;
use std::sync::atomic::{AtomicU64, Ordering};

/// A sentinel value placed in memory to detect heap tampering.
///
/// It allocates a value on the heap and checks if it has been modified
/// (e.g. by a memory scanner like Cheat Engine).
pub struct CanaryGuard {
    padding_before: Box<[AtomicU64; 16]>,
    location: Box<AtomicU64>,
    padding_after: Box<[AtomicU64; 16]>,
    expected_value: u64,
    xor_key: u64,
}

impl CanaryGuard {
    /// Initializes a new memory canary on the heap with buffer padding.
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let expected_value: u64 = rng.gen();
        let xor_key: u64 = rng.gen();

        // Obfuscate the value in memory
        let stored_value = expected_value ^ xor_key;
        let pad_val = xor_key;

        let padding_before = Box::new(std::array::from_fn(|_| AtomicU64::new(pad_val)));
        let location = Box::new(AtomicU64::new(stored_value));
        let padding_after = Box::new(std::array::from_fn(|_| AtomicU64::new(pad_val)));

        Self {
            padding_before,
            location,
            padding_after,
            expected_value,
            xor_key,
        }
    }

    /// Checks if the canary is intact. Returns `true` if intact, `false` if tampered.
    pub fn check(&self) -> bool {
        let stored_value = self.location.load(Ordering::Acquire);
        let decoded_value = stored_value ^ self.xor_key;
        if decoded_value != self.expected_value {
            return false;
        }

        // Check padding
        for pad in self.padding_before.iter() {
            if pad.load(Ordering::Acquire) != self.xor_key {
                return false;
            }
        }
        for pad in self.padding_after.iter() {
            if pad.load(Ordering::Acquire) != self.xor_key {
                return false;
            }
        }

        true
    }
}

//

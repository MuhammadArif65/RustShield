use rand::Rng;

/// A memory-obfuscated value wrapper that constantly rotates its encryption key
/// every time it is read or written. This prevents memory scanners like Cheat Engine
/// from finding the exact value in RAM.
pub struct Obfuscated32 {
    data: u32,
    key: u32,
}

impl Default for Obfuscated32 {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Obfuscated32 {
    /// Creates a new obfuscated value.
    pub fn new(value: u32) -> Self {
        let key = rand::thread_rng().gen::<u32>();
        Self {
            data: value ^ key,
            key,
        }
    }

    /// Reads the decrypted value and rotates the encryption key.
    pub fn get(&mut self) -> u32 {
        let value = self.data ^ self.key;

        // Rotate key to prevent static memory scanning
        let new_key = rand::thread_rng().gen::<u32>();
        self.data = value ^ new_key;
        self.key = new_key;

        value
    }

    /// Sets a new value and generates a fresh encryption key.
    pub fn set(&mut self, value: u32) {
        let new_key = rand::thread_rng().gen::<u32>();
        self.data = value ^ new_key;
        self.key = new_key;
    }

    /// Adds to the current value.
    pub fn add(&mut self, amount: u32) {
        let current = self.get();
        self.set(current.wrapping_add(amount));
    }

    /// Subtracts from the current value.
    pub fn sub(&mut self, amount: u32) {
        let current = self.get();
        self.set(current.wrapping_sub(amount));
    }
}

//

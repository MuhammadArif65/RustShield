# String Obfuscation in FerrumWard

FerrumWard supports compile-time string obfuscation through the `litcrypt` crate. This feature ensures that sensitive strings (like error messages, hardcoded API endpoints, or secret constants) do not appear in plain text within the compiled binary, making static analysis and reverse engineering significantly harder.

## Enabling the Feature

To enable string obfuscation, ensure that the `string-obfuscation` feature is active in your `Cargo.toml`. 
This is enabled by default in `ferrumward-core`.

```toml
[dependencies]
ferrumward-core = { version = "0.1", features = ["string-obfuscation"] }
```

You must also set the `LITCRYPT_ENCRYPT_KEY` environment variable before compiling your game. This key is used to encrypt the strings at compile time.

```bash
export LITCRYPT_ENCRYPT_KEY="your-super-secret-random-key"
cargo build --release
```

## How It Works

Under the hood, `ferrumward-core` uses the `litcrypt` procedural macros. 
When the feature is enabled, macros like `lc!()` replace the plaintext string with an encrypted byte array at compile time. At runtime, the string is decrypted in memory just before it is used.

If the `string-obfuscation` feature is **disabled**, the `lc!()` macro simply expands to the standard string literal, completely removing the overhead.

## Example Usage

In your game code, you can use the `lc!` macro exported by `litcrypt` (if you use it directly) or rely on FerrumWard's internal obfuscated strings.

```rust
use litcrypt::{lc, use_litcrypt};

// You must invoke this once per crate if you are using litcrypt directly
use_litcrypt!();

fn print_secret() {
    let secret = lc!("This string will not appear in the binary");
    println!("{}", secret);
}
```

## Security Considerations

1. **Memory Dump Risk**: While the string is obfuscated on disk, it exists in plaintext in memory after being decrypted. Advanced memory scanners might still find it.
2. **Key Security**: The `LITCRYPT_ENCRYPT_KEY` must never be hardcoded in your CI/CD scripts in plain text; use GitHub Secrets or equivalent secure injection methods.
3. **Performance Impact**: Decrypting strings at runtime incurs a tiny overhead. Avoid using obfuscated strings in extremely hot performance loops (like the inner render loop of your game engine).

<!-- -->

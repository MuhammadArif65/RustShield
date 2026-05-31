# AGENTS.md — FerrumWard AI Agent Instructions

## Constitution (MUST NOT BE VIOLATED)

1. **Security through cryptography, not obscurity** — SHA-256, Ed25519, AES-256-GCM.
2. **Zero runtime dependency** — no server, no telemetry, no cloud. 100% offline.
3. **Graceful failure** — `on_failure` callback MUST NOT reveal which check failed. Always return `FerrumWardError::TamperDetected` (generic).
4. **Developer experience matters** — integration = 1 line `add_plugins()` or 1 `[DllImport]`.
5. **No `unwrap()` / `expect()` / `panic!` / `println!`** in library code (allowed in tests only).
6. **No `unsafe`** except inside `#[cfg(target_os = "...")]` blocks with `// SAFETY: ...` comments.
7. **Dependency graph:** `ferrumward-core` MUST NOT depend on any other workspace crate.

## Pre-Commit Checklist

Run ALL of these before every commit. All must pass with zero warnings:

```bash
cargo fmt --all
cargo clippy --all-targets --features string-obfuscation -- -D warnings
cargo test --all
cargo build --all
# Linux only:
cargo clippy --all-targets --features tpm,string-obfuscation -- -D warnings
```

## Coding Patterns

### Platform-Specific Functions

```rust
pub fn platform_function() -> bool {
    #[cfg(target_os = "linux")]    { linux_impl() }
    #[cfg(target_os = "windows")]  { windows_impl() }
    #[cfg(target_os = "macos")]    { macos_impl() }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    { false }
}
```

### FFI Wrappers

```rust
#[no_mangle]
pub extern "C" fn ferrumward_function(/* ... */) -> i32 {
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { /* ... */ }));
    match r { Ok(Ok(_)) => 1, Ok(Err(_)) => 0, Err(_) => -1 }
}
```

### on_failure Callback (CORRECT)

```rust
if let Some(ref cb) = config.on_failure {
    cb(FerrumWardError::TamperDetected); // ALWAYS generic — never specific
}
```

## Architecture

- `ferrumward-core` — main library, zero workspace deps
- `ferrumward-cli` — CLI tool (clap v4)
- `ferrumward-ffi` — C FFI (cdylib + staticlib)
- `ferrumward-bevy` — Bevy plugin
- `ferrumward-godot` — Godot 4 GDExtension
- `ferrumward-mock-game` — Mock game for testing

## Feature Flags (ferrumward-core)

- `anti-debug`, `anti-vm`, `file-integrity`, `hardware-binding`, `license`, `crypto`, `checkpoint`, `canary` — all default
- `string-obfuscation` — optional (litcrypt)
- `tpm` — optional, Linux-only (tss-esapi)
- `full` — all features enabled


<!-- -->

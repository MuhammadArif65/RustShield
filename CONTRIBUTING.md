# Contributing to FerrumWard

Thank you for your interest in contributing to FerrumWard! This document provides guidelines for contributing.

## Code of Conduct

Be respectful. We are building security software — precision and professionalism matter.

## Getting Started

1. Fork the repository
2. Clone your fork
3. Set up the development environment:
   ```bash
   export LITCRYPT_ENCRYPT_KEY="YOUR_DEV_KEY"
   cargo build --workspace --all-features
   cargo test --workspace --all-features
   ```

## Development Rules (Constitution)

FerrumWard follows a strict constitution defined in [AGENTS.md](AGENTS.md). All contributions **must** adhere to:

1. **No `unwrap()`, `expect()`, or `panic!()` in library code** — Use `Result<T>` and return `FerrumWardError`.
2. **No `println!()` in library code** — All logging should be through the error/callback system.
3. **All `unsafe` blocks must have `// SAFETY:` comments** explaining why the operation is safe.
4. **All errors exposed externally must be `TamperDetected`** — Never reveal which specific check failed.
5. **No `std::process::exit()` in library code** — Only the game developer's callback should decide what to do.
6. **Zero external dependencies in core protection logic** — Crypto crates (`sha2`, `aes-gcm`, `ed25519-dalek`) are the exception.

## Pull Request Process

1. Create a feature branch: `git checkout -b feature/my-feature`
2. Ensure all tests pass: `LITCRYPT_ENCRYPT_KEY="SECRET" cargo test --workspace --all-features`
3. Ensure zero warnings: `cargo clippy --workspace --all-features`
4. Update documentation if needed
5. Submit a PR with a clear description

## Testing

- All new protection modules must include tests in `ferrumward-core/tests/`
- Security-critical code should include adversarial tests (see `brutal_tests.rs`)
- Run the full test suite before submitting:
  ```bash
  LITCRYPT_ENCRYPT_KEY="SECRET" cargo test --workspace --all-features
  ```

## Architecture

```
ferrumward-core/     → Core protection library (no framework dependencies)
ferrumward-cli/      → Developer CLI tool (keygen, manifest, license)
ferrumward-ffi/      → C-compatible FFI bridge (Unity, Unreal, custom engines)
ferrumward-bevy/     → Bevy engine plugin
ferrumward-godot/    → Godot 4 GDExtension
ferrumward-macros/   → Procedural macros
ferrumward-attacker/ → Red team simulation tool
```

## Questions?

Open a discussion on GitHub or reach out via the issue tracker.


<!-- -->

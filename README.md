# FerrumWard

![CI Status](https://img.shields.io/github/actions/workflow/status/MuhammadArif65/FerrumWard/ci.yml?branch=main)
![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)
![Rust Version](https://img.shields.io/badge/rustc-1.75+-lightgray.svg)

**FerrumWard** is a zero-dependency, modular, and extremely secure offline anti-piracy and game protection system written in Rust.

It provides out-of-the-box mechanisms to prevent debugging, virtualization, memory tampering, and unauthorized distribution through hardware-bound cryptography. FerrumWard is designed for offline games and ships with official wrappers for **Bevy**, **Godot**, **Unity**, and **Unreal Engine**.

---

### 📚 Documentation Hub
- 🚀 **[User Guide for Game Developers](USER_GUIDE.md)** - Start here! Step-by-step setup, integration, and distribution guide.
- 🧠 **[Architecture & Deep Dive](ARCHITECTURE.md)** - Learn how the Weighted Heuristic Scoring Engine and Chaotic Threading work under the hood.
- 🤝 **[Contributing](CONTRIBUTING.md)** - How to contribute to the project.
- 🛡️ **[Security Policy](SECURITY.md)** - Reporting vulnerabilities.

---

## Features

- 🛑 **Anti-Debug:** Detects `gdb`, `lldb`, `x64dbg`, CheatEngine, and API hooks natively on Windows, macOS, and Linux.
- 💻 **Anti-VM & Anti-Emulation:** Detects execution inside VirtualBox, VMware, QEMU, KVM, Wine, and Proton.
- 🧬 **Hardware Binding (HWID):** Generates robust, deterministic hardware fingerprints (UUID, CPU, RAM, Disk) preventing license sharing across machines.
- 🔑 **Cryptographic Licensing:** Employs Ed25519 digital signatures for offline activation keys.
- 🛡️ **Memory Integrity & Canary:** Uses atomic data canaries to detect memory tampering and memory scanners in real-time.
- ⏱️ **Time Tampering Guard:** Prevents players from modifying the system clock to bypass license expirations.
- 🕵️ **String Obfuscation:** Encrypts sensitive protection strings at compile time using `litcrypt`, leaving zero signatures for reverse engineers.

## Installation & Requirements

FerrumWard requires **Rust 1.75+**.

Clone the repository and build the workspace:
```bash
# Set your unique obfuscation key
export LITCRYPT_ENCRYPT_KEY="YOUR_SUPER_SECRET_KEY"

# Build all modules
cargo build --release --all --features full
```

## Workspace Structure

- `ferrumward-core`: The main protection logic (Zero external dependencies outside of `crypto`).
- `ferrumward-cli`: The developer tool for generating keys, manifests, and offline licenses.
- `ferrumward-ffi`: The C-compatible FFI layer for integration with Unity, Unreal Engine, and custom C/C++ games.
- `ferrumward-bevy`: Official plugin for the Bevy Engine.
- `ferrumward-godot`: Official GDExtension for Godot 4.
- `ferrumward-mock-game`: An end-to-end sandbox game showing how to integrate the protection.

## Usage Guide

### 1. Generate Developer Keys
Use the `ferrumward-cli` to generate your Ed25519 Keypair. This keypair is used to issue licenses to your players.

```bash
cargo run --bin ferrumward-cli -- keygen --output-dir ./keys
```
This generates `private.key` (keep this safe) and `public.key` (ship this with your game).
*Note: Both keys are stored as raw 32-byte Ed25519 binary files, not PEM/Base64.*

### 2. Generate Game Manifest
Create a cryptographic hash manifest of your game assets to detect modified files.

```bash
cargo run --bin ferrumward-cli -- manifest --target-dir ./assets --output manifest.json
```

### 3. Generate a Player License
When a user buys your game, they submit their Hardware ID (HWID). You generate a license bound to their machine.

```bash
cargo run --bin ferrumward-cli -- license \
    --hwid "PLAYER_HARDWARE_ID_HERE" \
    --private-key ./keys/private.key \
    --game-id "my-awesome-game" \
    --output player_license.sig
```

## Integration

### Bevy Engine
Add `ferrumward-bevy` to your Cargo.toml dependencies.

```rust
use bevy::prelude::*;
use ferrumward_bevy::FerrumWardPlugin;
use ferrumward_core::protection::ProtectionConfig;

fn main() {
    let config = ProtectionConfig {
        game_id: "my-awesome-game".to_string(),
        public_key: include_bytes!("../keys/public.key").to_vec(),
        license: Some(std::fs::read_to_string("player_license.sig").unwrap()),
        manifest_path: Some("manifest.json".into()),
        anti_debug: true,
        anti_vm: true,
        allow_proton: true,
        on_failure: Some(Box::new(|err| {
            println!("Game crashed! Reason: {:?}", err);
            std::process::exit(1);
        })),
    };

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FerrumWardPlugin::new(config))
        .run();
}
```

### Unity (C#)
Place the compiled `ferrumward_ffi.dll` in your `Assets/Plugins/` folder. Create a script to initialize the C-API.

```csharp
using System;
using System.Runtime.InteropServices;

public class FerrumWardIntegration : MonoBehaviour {
    [StructLayout(LayoutKind.Sequential)]
    public struct CProtectionConfig {
        public IntPtr game_id;
        public IntPtr public_key_ptr;
        public UIntPtr public_key_len;
        public IntPtr license;
        public IntPtr manifest_path;
        public bool anti_debug;
        public bool anti_vm;
        public IntPtr on_failure;
    }

    [DllImport("ferrumward_ffi", CallingConvention = CallingConvention.Cdecl)]
    public static extern int ferrumward_init(ref CProtectionConfig config);

    void Start() {
        // Initialize config and pass to ferrumward_init...
    }
}
```

### Unreal Engine (C++)
Include `ferrumward.h` and link the compiled `ferrumward_ffi` dynamic library in your `Build.cs`.

```cpp
#include "ferrumward.h"

void UMyGameInstance::Init() {
    Super::Init();
    
    CProtectionConfig Config;
    Config.game_id = "my-awesome-game";
    Config.anti_debug = true;
    Config.anti_vm = true;
    // ... initialize other fields
    
    if (ferrumward_init(&Config) != 1) {
        FPlatformMisc::RequestExit(true);
    }
}
```

## Best Practices for Production

If you intend to use FerrumWard for a real commercial game release, you **must** adhere to the following best practices to guarantee maximum security and compliance:

1. **Strict Key Management:** Never commit your `private.key` to any version control repository (GitHub/GitLab). Keep it completely isolated on a secure backend server that issues licenses.
2. **Release Profile Optimization:** Ensure that your game and the `ferrumward-loader` are compiled with the strongest release profile to strip symbols and prevent de-compilation:
   ```toml
   # Cargo.toml
   [profile.release]
   opt-level = 3
   lto = "fat"
   codegen-units = 1
   strip = "debuginfo"
   panic = "abort"
   ```
3. **Hardware Change Policy:** Because FerrumWard binds to MAC Addresses, CPUs, and Motherboards, legitimate players who upgrade their PC hardware will lose access to the game. It is highly recommended to build a web dashboard where players can reset their HWID binding (e.g., maximum 2 times per month) to avoid negative reviews.
4. **Legal & Privacy Policy (EULA):** FerrumWard collects hardware fingerprints locally. Ensure your game's EULA clearly states: *"This game collects hardware fingerprint data locally for Digital Rights Management (DRM) purposes. This data is not sold to third parties."* This is crucial to comply with GDPR and Steam's privacy guidelines.

## Security Notice
FerrumWard relies on Obscurity + Verification. While it dramatically raises the barrier for reverse engineering and piracy, no offline DRM is 100% unbreakable. It is highly recommended to pair FerrumWard with proper code obfuscation tools at the binary level.


<!-- -->


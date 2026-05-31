# Changelog

All notable changes to FerrumWard will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-05-30

### Added
- **Core Protection Modules**
  - Anti-Debug detection (Linux `TracerPid`, Windows `IsDebuggerPresent`, macOS `sysctl`)
  - Anti-VM detection (CPUID, DMI, Registry, Wine/Proton awareness)
  - Hardware ID binding with 7-component fingerprint (UUID, CPU, RAM, GPU, BIOS, MAC, Disk)
  - Ed25519 cryptographic license validation with offline activation
  - File integrity verification via SHA-256 manifest
  - Memory canary guards with atomic verification
  - Time tampering guard with drift detection
  - RWX memory scan for shellcode injection detection
  - Anti-injection module (detects foreign shared libraries)
  - Anti-suspend watchdog (detects thread freezing)
  - Anti-dump (ELF/PE header erasure from memory)
  - Hardware breakpoint detection
  - Parent process validation
  - Decoy honeypot variables (`player_health`, `player_gold`, `god_mode`)
  - Memory integrity checksumming
  - Secure encrypted storage (AES-256-GCM)
  - Variable obfuscator (XOR-based runtime value protection)

- **Neural Heuristic Scoring Engine**
  - 5-input, 4-hidden, 1-output MLP for multi-signal anomaly detection
  - Telemetry sensors: Page Faults, RDTSC Latency, Time Drift, Memory Entropy, Decoy Profiling
  - Entangled with Chaotic Hive-Mind threading for anti-removal

- **Chaotic Hive-Mind Threading**
  - Ephemeral phantom threads that spawn successors and die
  - Randomized check intervals (50-300ms) to prevent timing prediction

- **String Obfuscation**
  - Compile-time string encryption via `litcrypt` (feature-gated)
  - `rs_str!()` macro for transparent encrypted/plaintext string access

- **CLI Developer Tools**
  - `keygen` — Generate Ed25519 keypairs
  - `manifest` — Generate SHA-256 file integrity manifests
  - `license` — Issue hardware-bound offline licenses

- **Engine Integrations**
  - Bevy plugin (`ferrumward-bevy`)
  - Godot 4 GDExtension (`ferrumward-godot`)
  - C-compatible FFI bridge (`ferrumward-ffi`) for Unity/Unreal/custom engines

- **Red Team Simulation**
  - `ferrumward-attacker` binary with 4 attack modes: `ptrace`, `scan`, `inject`, `freeze`

- **Test Suite**
  - Burst hacking (concurrent protection init)
  - RWX memory injection detection
  - Time freeze simulation
  - Key forgery detection
  - Honeypot trap verification
  - License generation & validation
  - Hardware fingerprint determinism
  - Full protection lifecycle integration test


<!-- -->

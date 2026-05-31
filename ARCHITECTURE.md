# FerrumWard Architecture & Distribution Flow

This document provides a deep dive into the inner workings of FerrumWard, from its underlying architecture to the complete lifecycle of game distribution.

---

### 📚 Documentation Hub
- 🏠 **[Main README](README.md)** - Overview and quick start.
- 🚀 **[User Guide for Game Developers](USER_GUIDE.md)** - Step-by-step setup, integration, and distribution guide.

---

## 1. Project History & Philosophy

FerrumWard was born out of the necessity for a **zero-dependency, highly secure offline DRM** (Digital Rights Management) system. Many existing anti-piracy solutions either rely heavily on "always-online" requirements, which frustrate legitimate players, or they operate as invasive kernel-level drivers (Ring 0), which introduce massive security vulnerabilities and compatibility issues (e.g., preventing games from running on Linux/Steam Deck via Proton).

**The FerrumWard Philosophy:**
*   **Zero Dependency (User-Space Ring 3):** No kernel drivers. No external bloated libraries. Everything operates in user-space using Rust's zero-cost abstractions and standard OS APIs.
*   **No "Always-Online" Requirement:** Once activated, the game can be played offline forever, bound cryptographically to the user's hardware.
*   **Fail-Deadly, Silently:** If tampering is detected, the game does not show a helpful error message like "Debugger Detected". It simply crashes or triggers a generalized `TamperDetected` error, giving reverse engineers zero feedback on *what* caught them.
*   **Obscurity + Verification:** Combining compile-time string encryption (`litcrypt`), chaotic threading, and heuristic anomaly detection to make reverse engineering mathematically and practically exhausting.

## 2. Core Protection Mechanisms (The "How It Works")

FerrumWard does not rely on a single point of failure. It uses a multi-layered approach:

### 2. Weighted Heuristic Scoring Engine

Instead of relying purely on binary flags (e.g. `is_debugger_present == true`), FerrumWard utilizes a **Weighted Heuristic Scoring Engine** that evaluates multiple system signals simultaneously.

### Architecture (5 -> 4 -> 1)
- **Input Layer:** 5 sensory nodes capturing raw telemetry.
- **Hidden Layer:** 4 normalization nodes that weight the severity of the signals.
- **Output Layer:** 1 binary decision node (Tamper / Safe).

**The Sensors (Inputs):**
1.  **Page Fault Spikes:** Monitors `/proc/self/stat`. A massive, sudden spike in page faults usually indicates a memory scanner (like Cheat Engine) is aggressively reading the game's memory.
2.  **Instruction Latency (RDTSC):** Measures the CPU cycles taken to execute a tiny block of code. If it takes >1000 cycles, it implies a Hypervisor (VM) VM-exit occurred, or a debugger is stepping over the code.
3.  **Time Drift Variance:** Compares the execution time of the protection loop against real-world clock time. If a thread is suspended (e.g., via `SIGSTOP` or a debugger breakpoint), the time drift spikes.
4.  **Memory Entropy:** Samples the executable's `.text` segment. If the entropy is too high (close to 8.0), it suggests the memory contains encrypted or compressed injected shellcode.
5.  **Decoy Profiling:** Monitors the Honeypot (see below).

If the final sigmoid output exceeds the `0.85` threshold, the game is terminated.

### B. Decoy Honeypot (The Trap)
FerrumWard allocates highly attractive, completely fake variables in memory:
```rust
player_health: 100
player_gold: 999
god_mode: false
```
The actual game engine *never* reads or writes to these variables. However, a cheater using memory scanning tools will inevitably find them and attempt to freeze or modify them. The moment these variables change, the honeypot triggers the alarm, feeding a `1.0` (maximum suspicion) into the Neural Engine.

### C. Chaotic Hive-Mind Threading
Traditional anti-cheats run a single background thread. A reverse engineer can simply find this thread and call `SuspendThread` (Windows) or send `SIGSTOP` (Linux).
FerrumWard uses **Chaotic Threading**:
1. A thread spawns, waits for a random interval (50-300ms).
2. It performs the security checks.
3. It spawns the *next* thread.
4. It immediately kills itself.

There is no permanent anti-cheat thread to suspend. If an attacker kills the current thread before it spawns the next one, the protection loop dies—but this is tied to game asset decryption, meaning the game itself will break shortly after. Thread names are randomized (`rs_XXXXXXXX`) to evade simple `grep` or `htop` filtering.

### D. Hardware Binding & Cryptography
To prevent a user from buying the game and sending the license to their friends, the license is cryptographically bound to a 7-component Hardware ID (HWID):
*   Machine UUID (`/etc/machine-id` or Windows Registry)
*   CPU string
*   Total RAM (rounded to the nearest 4GB to prevent issues with minor memory reservations)
*   GPU string
*   BIOS UUID
*   MAC Address (Primary active network interface)
*   Disk Serial Number

This data is hashed into a deterministic string.

## 3. The Game Distribution Flow (End-to-End)

Here is exactly how a game developer uses FerrumWard from compiling the game to the player launching it.

### Phase 1: The Developer Preparation
1.  **Generate Developer Keys:** The developer runs `ferrumward-cli keygen`. This generates an Ed25519 `private.key` (kept strictly on the developer's secure backend/server) and a `public.key` (embedded into the game binary).
2.  **Build the Game:** The developer compiles the game (e.g., in Unity, Bevy, or Godot) with the FerrumWard plugin included. The `public.key` is compiled *into* the binary.
3.  **Generate File Manifest:** The developer runs `ferrumward-cli manifest --target-dir ./game_data`. This creates `manifest.json`, containing the SHA-256 hashes of every game asset (models, textures, scripts).
4.  **Distribution:** The developer uploads the Game Binary, Game Assets, and `manifest.json` to Steam, Itch.io, or their own website.

### Phase 2: The Player Purchase & Activation
1.  **Player Buys the Game:** The player purchases the game on the developer's website.
2.  **HWID Generation:** The player runs the game for the first time. The game detects no license. It calculates the player's HWID (e.g., `MACHINE-A1B2C3D4`) and displays it on the screen.
3.  **License Request:** The player copies this HWID and pastes it into the developer's website dashboard.
4.  **License Issuance:** The developer's backend server takes the HWID and the Game ID, and signs them using the **`private.key`**. It generates a `player_license.sig` (a base64 string).
5.  **Activation:** The player downloads `player_license.sig` and places it in the game folder.

### Phase 3: The Runtime Validation (Every time the game launches)
1.  **Launch:** The player double-clicks the game executable.
2.  **Environment Check:** FerrumWard immediately scans for debuggers (TracerPid), VMs, and RWX memory injections.
3.  **Integrity Check:** FerrumWard hashes the local game assets and compares them against `manifest.json`. If a modder altered a file, the game crashes.
4.  **License Validation:**
    *   FerrumWard calculates the current machine's HWID.
    *   It reads `player_license.sig`.
    *   Using the embedded `public.key`, it verifies that the signature is mathematically valid and was signed by the developer.
    *   It decrypts the license to ensure the embedded HWID matches the *current* machine's HWID. If the player copied the game to a different PC, the HWIDs won't match, and the game closes.
5.  **Game Starts:** The game engine initializes.
6.  **Active Protection:** The Chaotic Hive-Mind threading begins, running the Neural Heuristic Engine in the background 3-20 times per second to ensure no cheating tools are attached during gameplay.


<!-- -->


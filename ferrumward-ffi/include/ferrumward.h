#ifndef FERRUMWARD_H
#define FERRUMWARD_H

#include <stdint.h>
#if __STDC_VERSION__ >= 199901L || defined(__cplusplus)
#include <stdbool.h>
#else
#ifndef bool
#define bool int
#define true 1
#define false 0
#endif
#endif
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * C-compatible representation of the FerrumWard protection configuration.
 */
typedef struct {
    /** 
     * Unique identifier for the game. Must be a null-terminated string.
     */
    const char* game_id;
    
    /** 
     * Pointer to the Ed25519 public key bytes.
     */
    const uint8_t* public_key_ptr;
    
    /** 
     * Length of the public key (should be 32 bytes).
     */
    size_t public_key_len;
    
    /** 
     * Offline license content. Nullable if no license is required yet.
     */
    const char* license;
    
    /** 
     * Path to the manifest.json file for integrity checking. Nullable.
     */
    const char* manifest_path;
    
    /** 
     * Set to true to enable the Anti-Debugging module.
     */
    bool anti_debug;
    
    /** 
     * Set to true to enable the Anti-VM/Hypervisor module.
     */
    bool anti_vm;
    
    /** 
     * Callback function invoked when tampering is detected.
     * Can be NULL. If NULL, the engine logs the error but does not exit.
     */
    void (*on_failure)();
    
} CProtectionConfig;

/**
 * Initializes the FerrumWard protection engine.
 * 
 * @param config Pointer to the CProtectionConfig struct.
 * @return 1 on success, 0 on failure (tampering detected), -1 on invalid arguments or panic.
 */
int ferrumward_init(const CProtectionConfig* config);

/**
 * Triggers a manual, randomized protection checkpoint.
 * Call this randomly during your game loop or level loads.
 * 
 * @return 1 on success (clean), 0 on failure (tampering detected), -1 on panic.
 */
int ferrumward_run_checkpoint();

#ifdef __cplusplus
}
#endif

#endif // FERRUMWARD_H

// 

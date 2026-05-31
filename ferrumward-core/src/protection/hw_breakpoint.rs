#![allow(unused_imports)]
use crate::error::{Result, FerrumWardError};

/// Checks if hardware breakpoints (DR0-DR3) are active on the current thread.
pub fn assert_no_hardware_breakpoints() -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        check_windows()?;
    }

    // On Linux/macOS, checking hardware breakpoints without ptrace (which we can't use on self)
    // or signals is extremely difficult without external dependencies or assembly.
    // For now, we'll keep it as a no-op on non-Windows platforms.

    Ok(())
}

#[cfg(target_os = "windows")]
fn check_windows() -> Result<()> {
    use std::os::raw::c_void;

    // We define the minimal CONTEXT structure.
    // Since Windows has x86 and x64, we'll align to x64 layout for simplicity,
    // assuming a 64-bit target, but to be robust we just check the return status.
    // To remain truly robust across archs without winapi, we use a large enough byte array.

    #[repr(C, align(16))]
    struct ContextAlign16([u8; 1232]); // CONTEXT for x64 is 1232 bytes

    extern "system" {
        fn GetCurrentThread() -> *mut c_void;
        fn GetThreadContext(hThread: *mut c_void, lpContext: *mut c_void) -> i32;
    }

    #[cfg(target_arch = "x86_64")]
    const CONTEXT_DEBUG_REGISTERS: u32 = 0x00100010;
    #[cfg(target_arch = "x86")]
    const CONTEXT_DEBUG_REGISTERS: u32 = 0x00010010;

    // SAFETY: [Rule 2 Exception] We are querying our own thread context.
    unsafe {
        let h_thread = GetCurrentThread();
        let mut ctx = ContextAlign16([0; 1232]);

        // The ContextFlags is the first u32 in x86, but it's at offset 0x30 in x64.
        // We will just set the first 64 bytes to CONTEXT_DEBUG_REGISTERS, hoping it hits the flag.
        // A safer way is to use a properly defined struct, but for zero-deps this is tricky.
        // Let's use a simpler approach: many anti-cheats just check the DR registers via SEH.
        // Since we can't easily do SEH in pure Rust, we'll trust the offset 0x30 for x64.
        #[cfg(target_arch = "x86_64")]
        {
            let flags_ptr = ctx.0.as_mut_ptr().add(0x30) as *mut u32;
            *flags_ptr = CONTEXT_DEBUG_REGISTERS;

            if GetThreadContext(h_thread, ctx.0.as_mut_ptr() as *mut c_void) != 0 {
                // DR0 is at 0x48, DR1 at 0x50, DR2 at 0x58, DR3 at 0x60
                let dr0 = *(ctx.0.as_ptr().add(0x48) as *const u64);
                let dr1 = *(ctx.0.as_ptr().add(0x50) as *const u64);
                let dr2 = *(ctx.0.as_ptr().add(0x58) as *const u64);
                let dr3 = *(ctx.0.as_ptr().add(0x60) as *const u64);

                if dr0 != 0 || dr1 != 0 || dr2 != 0 || dr3 != 0 {
                    return Err(FerrumWardError::TamperDetected);
                }
            }
        }
        #[cfg(target_arch = "x86")]
        {
            let flags_ptr = ctx.0.as_mut_ptr() as *mut u32; // ContextFlags is first in x86
            *flags_ptr = CONTEXT_DEBUG_REGISTERS;

            if GetThreadContext(h_thread, ctx.0.as_mut_ptr() as *mut c_void) != 0 {
                // DR0 is at 0x04, DR1 at 0x08, DR2 at 0x0C, DR3 at 0x10
                let dr0 = *(ctx.0.as_ptr().add(0x04) as *const u32);
                let dr1 = *(ctx.0.as_ptr().add(0x08) as *const u32);
                let dr2 = *(ctx.0.as_ptr().add(0x0C) as *const u32);
                let dr3 = *(ctx.0.as_ptr().add(0x10) as *const u32);

                if dr0 != 0 || dr1 != 0 || dr2 != 0 || dr3 != 0 {
                    return Err(FerrumWardError::TamperDetected);
                }
            }
        }
    }

    Ok(())
}

//

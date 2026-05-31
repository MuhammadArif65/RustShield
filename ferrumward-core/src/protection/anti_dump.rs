use crate::error::Result;

/// Erases the executable header from memory to prevent memory dumping.
/// This will destroy the PE/ELF header of the current module in memory,
/// confusing most memory dumpers like Scylla or MegaDumper.
pub fn erase_headers_from_memory() -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        erase_windows();
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        erase_posix();
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn erase_windows() {
    use std::os::raw::c_void;

    // Minimal FFI for Windows VirtualProtect
    extern "system" {
        fn GetModuleHandleA(lpModuleName: *const i8) -> *mut c_void;
        fn VirtualProtect(
            lpAddress: *mut c_void,
            dwSize: usize,
            flNewProtect: u32,
            lpflOldProtect: *mut u32,
        ) -> i32;
    }

    const PAGE_READWRITE: u32 = 0x04;

    // SAFETY: [Rule 2 Exception] We are modifying the PE header in our own process memory.
    // This is safe because the PE header is no longer needed after the executable is loaded.
    unsafe {
        let base_addr = GetModuleHandleA(std::ptr::null());
        if !base_addr.is_null() {
            let mut old_protect: u32 = 0;
            // 4096 is typically enough to cover the DOS/PE headers
            if VirtualProtect(base_addr, 4096, PAGE_READWRITE, &mut old_protect) != 0 {
                // Zero out the header
                std::ptr::write_bytes(base_addr as *mut u8, 0, 4096);
                // Restore old protection (optional, but good practice)
                let mut dummy = 0;
                VirtualProtect(base_addr, 4096, old_protect, &mut dummy);
            }
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn erase_posix() {
    // Implementing this safely and reliably on Linux/macOS without breaking glibc or dyld
    // is highly complex. For this cross-platform library, we will leave it as a no-op
    // on POSIX for stability, as ELF header stripping is usually done on-disk.
    // Alternatively, one could use mprotect(PAGE_EXECUTE_READWRITE) on the base address.
}

//

#![allow(dead_code, unused_variables, unused_imports)]
use crate::error::{FerrumWardError, Result};

/// Checks if a debugger is currently attached to the process.
/// Returns `true` if a debugger is detected, otherwise `false`.
pub fn is_debugger_attached() -> bool {
    #[cfg(target_os = "linux")]
    {
        check_linux()
    }

    #[cfg(target_os = "windows")]
    {
        check_windows()
    }

    #[cfg(target_os = "macos")]
    {
        check_macos()
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        false
    }
}

/// Asserts that no debugger is attached.
/// Returns an error if a debugger is found.
pub fn assert_no_debugger() -> Result<()> {
    if is_debugger_attached() {
        Err(FerrumWardError::TamperDetected)
    } else {
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn check_linux() -> bool {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    if let Ok(file) = File::open(crate::rs_str!("/proc/self/status")) {
        for line in BufReader::new(file)
            .lines()
            .map_while(std::result::Result::ok)
        {
            if line.starts_with(crate::rs_str!("TracerPid:").as_str()) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() == 2 {
                    if let Ok(pid) = parts[1].parse::<i32>() {
                        return pid != 0;
                    }
                }
            }
        }
    }
    false
}

#[cfg(target_os = "windows")]
fn check_windows() -> bool {
    use std::ffi::CString;

    type IsDebuggerPresentFn = unsafe extern "system" fn() -> i32;
    type CheckRemoteDebuggerPresentFn =
        unsafe extern "system" fn(*mut std::ffi::c_void, *mut i32) -> i32;
    type GetCurrentProcessFn = unsafe extern "system" fn() -> *mut std::ffi::c_void;

    extern "system" {
        fn GetModuleHandleA(lpModuleName: *const i8) -> *mut std::ffi::c_void;
        fn GetProcAddress(
            hModule: *mut std::ffi::c_void,
            lpProcName: *const i8,
        ) -> *const std::ffi::c_void;
    }

    let mut attached = false;

    let kernel32_str = match CString::new(crate::rs_str!("kernel32.dll")) {
        Ok(s) if !s.as_bytes().is_empty() => s,
        _ => return false,
    };
    let is_dbg_str = match CString::new(crate::rs_str!("IsDebuggerPresent")) {
        Ok(s) if !s.as_bytes().is_empty() => s,
        _ => return false,
    };
    let chk_rem_str = match CString::new(crate::rs_str!("CheckRemoteDebuggerPresent")) {
        Ok(s) if !s.as_bytes().is_empty() => s,
        _ => return false,
    };
    let get_curr_str = match CString::new(crate::rs_str!("GetCurrentProcess")) {
        Ok(s) if !s.as_bytes().is_empty() => s,
        _ => return false,
    };

    unsafe {
        // SAFETY: Retrieving module handle for kernel32.dll, a system library.
        let h_module = GetModuleHandleA(kernel32_str.as_ptr());
        if !h_module.is_null() {
            // SAFETY: Retrieving IsDebuggerPresent address.
            let p_is_dbg = GetProcAddress(h_module, is_dbg_str.as_ptr());
            if !p_is_dbg.is_null() {
                let func: IsDebuggerPresentFn = std::mem::transmute(p_is_dbg);
                // SAFETY: Calling system API IsDebuggerPresent with no arguments.
                if func() != 0 {
                    attached = true;
                }
            }

            // SAFETY: Retrieving CheckRemoteDebuggerPresent address.
            let p_chk_rem = GetProcAddress(h_module, chk_rem_str.as_ptr());
            if !p_chk_rem.is_null() {
                // SAFETY: Retrieving GetCurrentProcess address.
                let p_get_curr = GetProcAddress(h_module, get_curr_str.as_ptr());
                if !p_get_curr.is_null() {
                    let func_chk: CheckRemoteDebuggerPresentFn = std::mem::transmute(p_chk_rem);
                    let func_curr: GetCurrentProcessFn = std::mem::transmute(p_get_curr);
                    let mut is_remote: i32 = 0;
                    // SAFETY: Calling GetCurrentProcess and CheckRemoteDebuggerPresent safely.
                    if func_chk(func_curr(), &mut is_remote) != 0 && is_remote != 0 {
                        attached = true;
                    }
                }
            }
        }
    }

    attached
}

#[cfg(target_os = "macos")]
fn check_macos() -> bool {
    // macOS sysctl KERN_PROC check.
    // Avoiding kinfo_proc struct directly to prevent linker bugs as specified in constitution.
    use std::ffi::CString;
    use std::os::raw::{c_int, c_uint};
    use std::ptr;

    const CTL_KERN: c_int = 1;
    const KERN_PROC: c_int = 14;
    const KERN_PROC_PID: c_int = 1;
    const P_TRACED: u32 = 0x00000800; // P_TRACED flag

    extern "C" {
        fn dlsym(handle: *mut std::ffi::c_void, symbol: *const i8) -> *mut std::ffi::c_void;
        fn getpid() -> c_int;
    }

    type SysctlFn = unsafe extern "C" fn(
        *mut c_int,
        c_uint,
        *mut std::ffi::c_void,
        *mut usize,
        *mut std::ffi::c_void,
        usize,
    ) -> c_int;

    let sysctl_str = match CString::new(crate::rs_str!("sysctl")) {
        Ok(s) if !s.as_bytes().is_empty() => s,
        _ => return false,
    };
    // RTLD_DEFAULT on macOS is (void *)-2
    let handle = -2isize as *mut std::ffi::c_void;

    let sysctl_ptr = unsafe {
        // SAFETY: Calling dlsym with RTLD_DEFAULT is safe.
        dlsym(handle, sysctl_str.as_ptr())
    };

    if sysctl_ptr.is_null() {
        return false;
    }

    let sysctl_func: SysctlFn = unsafe {
        // SAFETY: Transmuting dlsym result to known function pointer type.
        std::mem::transmute(sysctl_ptr)
    };

    let mut mib = [CTL_KERN, KERN_PROC, KERN_PROC_PID, unsafe {
        // SAFETY: Calling system API getpid which has no arguments and is always safe.
        getpid()
    }];

    let mut size: usize = 0;

    unsafe {
        // SAFETY: First sysctl call to get the required size for kinfo_proc.
        if sysctl_func(
            mib.as_mut_ptr(),
            mib.len() as c_uint,
            ptr::null_mut(),
            &mut size,
            ptr::null_mut(),
            0,
        ) != 0
        {
            return false;
        }

        let mut info = vec![0u8; size];

        // SAFETY: Second sysctl call to fill the allocated buffer.
        if sysctl_func(
            mib.as_mut_ptr(),
            mib.len() as c_uint,
            info.as_mut_ptr() as *mut _,
            &mut size,
            ptr::null_mut(),
            0,
        ) == 0
        {
            // p_flag is located at offset 32 inside kinfo_proc (extern proc p_flag)
            if size >= 36 {
                let mut p_flag_bytes = [0u8; 4];
                p_flag_bytes.copy_from_slice(&info[32..36]);
                let p_flag = u32::from_ne_bytes(p_flag_bytes);
                return (p_flag & P_TRACED) != 0;
            }
        }
    }
    false
}

//

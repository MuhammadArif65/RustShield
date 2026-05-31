#![allow(dead_code, unused_variables, unused_imports)]
use crate::error::{FerrumWardError, Result};

/// Checks if the process is running inside a virtual machine.
/// Returns `true` if a VM is detected, otherwise `false`.
pub fn is_virtual_machine(allow_proton: bool) -> bool {
    #[cfg(target_os = "linux")]
    {
        check_linux(allow_proton)
    }

    #[cfg(target_os = "windows")]
    {
        check_windows(allow_proton)
    }

    #[cfg(target_os = "macos")]
    {
        check_macos(allow_proton)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        false
    }
}

/// Asserts that the process is not running inside a virtual machine.
/// Returns an error if a VM is found.
pub fn assert_no_virtual_machine(allow_proton: bool) -> Result<()> {
    if is_virtual_machine(allow_proton) {
        Err(FerrumWardError::TamperDetected)
    } else {
        Ok(())
    }
}

/// Checks if the given string contains a known VM identifier but not a whitelisted one.
fn contains_vm_signature(info: &str, allow_proton: bool) -> bool {
    let lower = info.to_lowercase();

    // Whitelist check first
    if allow_proton {
        if lower.contains(crate::rs_str!("wine").as_str())
            || lower.contains(crate::rs_str!("proton").as_str())
            || lower.contains(crate::rs_str!("valve").as_str())
            || lower.contains(crate::rs_str!("steamdeck").as_str())
        {
            return false;
        }
    }

    // Blacklist check
    lower.contains(crate::rs_str!("qemu").as_str())
        || lower.contains(crate::rs_str!("virtualbox").as_str())
        || lower.contains(crate::rs_str!("vmware").as_str())
        || lower.contains(crate::rs_str!("kvm").as_str())
        || lower.contains(crate::rs_str!("vbox").as_str())
        || lower.contains(crate::rs_str!("parallels").as_str())
        || lower.contains(crate::rs_str!("bhyve").as_str())
}

#[cfg(target_os = "linux")]
fn check_linux(allow_proton: bool) -> bool {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    // Check CPU info
    if let Ok(file) = File::open(crate::rs_str!("/proc/cpuinfo")) {
        for line in BufReader::new(file)
            .lines()
            .map_while(std::result::Result::ok)
        {
            if contains_vm_signature(&line, allow_proton) {
                return true;
            }
        }
    }

    // Check DMI sysfs (vendor/product name)
    if let Ok(vendor) = std::fs::read_to_string(crate::rs_str!("/sys/class/dmi/id/sys_vendor")) {
        if contains_vm_signature(&vendor, allow_proton) {
            return true;
        }
    }
    if let Ok(product) = std::fs::read_to_string(crate::rs_str!("/sys/class/dmi/id/product_name")) {
        if contains_vm_signature(&product, allow_proton) {
            return true;
        }
    }

    false
}

#[cfg(target_os = "windows")]
fn check_windows(allow_proton: bool) -> bool {
    use std::process::Command;

    // We use command line to query the registry since we avoid adding unapproved dependencies.
    if let Ok(output) = Command::new(crate::rs_str!("reg"))
        .args([
            crate::rs_str!("query"),
            crate::rs_str!(r"HKLM\Hardware\Description\System\BIOS"),
            crate::rs_str!("/v"),
            crate::rs_str!("SystemManufacturer"),
        ])
        .output()
    {
        if let Ok(text) = String::from_utf8(output.stdout) {
            if contains_vm_signature(&text, allow_proton) {
                return true;
            }
        }
    }

    if let Ok(output) = Command::new(crate::rs_str!("reg"))
        .args([
            crate::rs_str!("query"),
            crate::rs_str!(r"HKLM\Hardware\Description\System\BIOS"),
            crate::rs_str!("/v"),
            crate::rs_str!("SystemProductName"),
        ])
        .output()
    {
        if let Ok(text) = String::from_utf8(output.stdout) {
            if contains_vm_signature(&text, allow_proton) {
                return true;
            }
        }
    }

    false
}

#[cfg(target_os = "macos")]
fn check_macos(allow_proton: bool) -> bool {
    use std::ffi::CString;
    use std::os::raw::c_int;
    use std::ptr;

    extern "C" {
        fn dlsym(handle: *mut std::ffi::c_void, symbol: *const i8) -> *mut std::ffi::c_void;
    }

    type SysctlBynameFn = unsafe extern "C" fn(
        *const i8,
        *mut std::ffi::c_void,
        *mut usize,
        *mut std::ffi::c_void,
        usize,
    ) -> c_int;

    let sysctl_str = match CString::new(crate::rs_str!("sysctlbyname")) {
        Ok(s) if !s.as_bytes().is_empty() => s,
        _ => return false,
    };
    let hw_model_str = match CString::new(crate::rs_str!("hw.model")) {
        Ok(s) if !s.as_bytes().is_empty() => s,
        _ => return false,
    };

    let handle = -2isize as *mut std::ffi::c_void;
    // SAFETY: Calling dlsym from libc is safe. The string is null-terminated and valid.
    let sysctl_ptr = unsafe { dlsym(handle, sysctl_str.as_ptr()) };
    if sysctl_ptr.is_null() {
        return false;
    }

    // SAFETY: We verified sysctl_ptr is not null. It points to the sysctlbyname function.
    let sysctl_func: SysctlBynameFn = unsafe { std::mem::transmute(sysctl_ptr) };

    let mut size: usize = 0;
    // SAFETY: We provide valid pointers and sizes as expected by sysctlbyname.
    unsafe {
        if sysctl_func(
            hw_model_str.as_ptr(),
            ptr::null_mut(),
            &mut size,
            ptr::null_mut(),
            0,
        ) != 0
        {
            return false;
        }

        let mut info = vec![0u8; size];
        if sysctl_func(
            hw_model_str.as_ptr(),
            info.as_mut_ptr() as *mut _,
            &mut size,
            ptr::null_mut(),
            0,
        ) == 0
        {
            // Null terminated, but String::from_utf8 handles the slice. We can trim nulls later or just check contains.
            if let Ok(text) = String::from_utf8(info) {
                let clean_text = text.trim_matches(char::from(0));
                if contains_vm_signature(clean_text, allow_proton) {
                    return true;
                }
            }
        }
    }
    false
}

//

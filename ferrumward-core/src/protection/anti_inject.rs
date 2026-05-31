use crate::error::{FerrumWardError, Result};

/// Checks for suspicious injected libraries (DLLs/SOs).
pub fn assert_no_injected_modules() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        check_linux()?;
    }

    #[cfg(target_os = "windows")]
    {
        check_windows()?;
    }

    #[cfg(target_os = "macos")]
    {
        check_macos()?;
    }

    Ok(())
}

fn contains_bad_module(module_name: &str) -> bool {
    let lower = module_name.to_lowercase();

    let blacklist = [
        crate::rs_str!("cheatengine"),
        crate::rs_str!("speedhack"),
        crate::rs_str!("inject"),
        crate::rs_str!("hook"),
        crate::rs_str!("x64dbg"),
    ];

    for bad in &blacklist {
        if lower.contains(&bad.to_lowercase()) {
            return true;
        }
    }
    false
}

#[cfg(target_os = "linux")]
fn check_linux() -> Result<()> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    if let Ok(file) = File::open(crate::rs_str!("/proc/self/maps")) {
        for line in BufReader::new(file)
            .lines()
            .map_while(std::result::Result::ok)
        {
            if contains_bad_module(&line) {
                return Err(FerrumWardError::TamperDetected);
            }
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn check_windows() -> Result<()> {
    // Note: To remain zero-dependency (without heavy winapi crates),
    // we query the loaded modules via tasklist if available.
    use std::process::Command;

    let pid = std::process::id();
    if let Ok(output) = Command::new(crate::rs_str!("tasklist"))
        .args([
            crate::rs_str!("/m"),
            crate::rs_str!("/fi"),
            format!("{} {}", crate::rs_str!("PID eq"), pid),
        ])
        .output()
    {
        if let Ok(text) = String::from_utf8(output.stdout) {
            if contains_bad_module(&text) {
                return Err(FerrumWardError::TamperDetected);
            }
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn check_macos() -> Result<()> {
    // macOS has vmmap, but it's slow. We'll use vmmap for the current PID.
    use std::process::Command;

    let pid = std::process::id();
    if let Ok(output) = Command::new(crate::rs_str!("vmmap"))
        .arg(pid.to_string())
        .output()
    {
        if let Ok(text) = String::from_utf8(output.stdout) {
            if contains_bad_module(&text) {
                return Err(FerrumWardError::TamperDetected);
            }
        }
    }
    Ok(())
}

//

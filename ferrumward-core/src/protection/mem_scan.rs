#![allow(dead_code, unused_variables, unused_imports)]
use crate::error::{Result, FerrumWardError};

#[cfg(target_os = "linux")]
pub fn scan_for_rwx_memory() -> Result<()> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(crate::rs_str!("/proc/self/maps"))
        .map_err(|_| FerrumWardError::TamperDetected)?;

    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.map_err(|_| FerrumWardError::TamperDetected)?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let perms = parts[1];
            // If the memory segment is marked Read, Write, and Execute (rwxp or rwx-),
            // it is highly suspicious and often indicative of injected shellcode or cheat engines.
            if perms.starts_with(crate::rs_str!("rwx").as_str()) {
                return Err(FerrumWardError::TamperDetected);
            }
        }
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn scan_for_rwx_memory() -> Result<()> {
    // Stub for non-Linux OS. In Windows, we would use VirtualQuery to check for PAGE_EXECUTE_READWRITE.
    Ok(())
}

//

use crate::error::{FerrumWardError, Result};
use sysinfo::System;

/// Asserts that the game was launched by a legitimate process and not a debugger.
pub fn assert_valid_parent_process() -> Result<()> {
    // Only check if sysinfo is supported on the target OS
    let mut sys = System::new();
    // Refresh only process lists to save time
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let current_pid = sysinfo::get_current_pid().map_err(|_| FerrumWardError::TamperDetected)?;

    if let Some(process) = sys.process(current_pid) {
        if let Some(parent_pid) = process.parent() {
            if let Some(parent_process) = sys.process(parent_pid) {
                let parent_name = parent_process.name().to_string_lossy().to_lowercase();

                // Blacklist of known debuggers and suspicious launchers
                let blacklist = [
                    crate::rs_str!("x64dbg"),
                    crate::rs_str!("x32dbg"),
                    crate::rs_str!("gdb"),
                    crate::rs_str!("lldb"),
                    crate::rs_str!("cheatengine"),
                    crate::rs_str!("ollydbg"),
                    crate::rs_str!("ida"),
                    crate::rs_str!("ida64"),
                    crate::rs_str!("radare2"),
                    crate::rs_str!("devenv"), // Visual Studio
                    crate::rs_str!("windbg"),
                ];

                for bad_parent in &blacklist {
                    let bad_lower = bad_parent.to_lowercase();
                    if parent_name.contains(&bad_lower) {
                        return Err(FerrumWardError::TamperDetected);
                    }
                }
            }
        }
    }

    Ok(())
}

//

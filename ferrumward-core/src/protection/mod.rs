#[cfg(feature = "anti-debug")]
pub mod anti_debug;
pub mod anti_dump;
pub mod anti_inject;
pub mod anti_suspend;
#[cfg(feature = "anti-vm")]
pub mod anti_vm;
#[cfg(feature = "canary")]
pub mod canary;
pub mod chaotic_thread;
#[cfg(feature = "checkpoint")]
pub mod checkpoint;
pub mod decoy_honeypot;
pub mod heuristic_ai;
pub mod hw_breakpoint;
pub mod integrity;
pub mod kinematic_anomaly;
pub mod mem_integrity;
pub mod mem_scan;
pub mod parent_process;
pub mod secure_storage;
pub mod self_check;
pub mod time_guard;
pub mod var_obfuscator;

pub use integrity::{protect, ProtectionConfig};
pub use mem_scan::scan_for_rwx_memory;
pub use self_check::{init_self_check, verify_self_check};
pub use time_guard::{check_time_tampering, init_time_guard};

//

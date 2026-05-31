#![allow(dead_code, unused_variables, unused_imports)]
use crate::error::{FerrumWardError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::process::Command;
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HwidProfile {
    pub machine: String,
    pub cpu: String,
    pub ram: String,
    pub gpu: String,
    pub bios: String,
    pub disk: String,
}

impl HwidProfile {
    /// Hashes a single value
    fn hash_value(val: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(val.as_bytes());
        let hex = format!("{:x}", hasher.finalize());
        let res = hex.get(..32).unwrap_or(&hex).to_string();
        println!("HASHING: {:?} -> {}", val, res);
        res
    }

    /// Convert the raw values to a hashed profile
    pub fn to_hashed(&self) -> Self {
        Self {
            machine: Self::hash_value(&self.machine),
            cpu: Self::hash_value(&self.cpu),
            ram: Self::hash_value(&self.ram),
            gpu: Self::hash_value(&self.gpu),
            bios: Self::hash_value(&self.bios),
            disk: Self::hash_value(&self.disk),
        }
    }

    /// Compute similarity score (0 to 7 matches)
    pub fn match_score(&self, other: &Self) -> usize {
        let mut score = 0;
        if self.machine == other.machine {
            score += 1;
        }
        if self.cpu == other.cpu {
            score += 1;
        }
        if self.ram == other.ram {
            score += 1;
        }
        if self.gpu == other.gpu {
            score += 1;
        }
        if self.bios == other.bios {
            score += 1;
        }
        if self.disk == other.disk {
            score += 1;
        }
        score
    }
}

pub fn get_hwid_profile() -> Result<HwidProfile> {
    // 1. Get Machine UUID
    println!(
        "IS OBFUSCATION ON? {}",
        cfg!(feature = "string-obfuscation")
    );
    let machine = machine_uid::get().unwrap_or_else(|_| crate::rs_str!("unknown_machine"));

    // 2. CPU and RAM
    let mut sys = System::new();
    sys.refresh_cpu_all();
    sys.refresh_memory();

    let cpu = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| crate::rs_str!("unknown_cpu"));

    let ram_bytes = sys.total_memory();
    let ram_gb = ram_bytes as f64 / 1_073_741_824.0;
    let ram_gb_rounded = (ram_gb / 4.0).round() as u64 * 4;
    let ram = format!("{}GB", ram_gb_rounded);

    // 3. GPU
    println!("MACRO GPU: {:?}", crate::rs_str!("unknown_gpu"));
    let gpu = get_gpu_id().unwrap_or_else(|| crate::rs_str!("unknown_gpu"));

    // 4. BIOS UUID
    let bios = get_bios_uuid().unwrap_or_else(|| crate::rs_str!("unknown_bios"));

    // 5. Disk Serial
    let disk = get_disk_serial().unwrap_or_else(|| crate::rs_str!("unknown_disk"));

    Ok(HwidProfile {
        machine,
        cpu,
        ram,
        gpu,
        bios,
        disk,
    })
}

/// Helper to serialize the hashed HWID to a Base64 string for the License
pub fn get_hardware_id() -> Result<String> {
    let profile = get_hwid_profile()?.to_hashed();
    let json = serde_json::to_string(&profile)
        .map_err(|e| FerrumWardError::CryptoError(format!("Serialization error: {}", e)))?;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    Ok(URL_SAFE_NO_PAD.encode(json))
}

// OS Specific implementations
#[cfg(target_os = "windows")]
fn get_gpu_id() -> Option<String> {
    let output = Command::new(crate::rs_str!("powershell"))
        .args([
            crate::rs_str!("-NoProfile"),
            crate::rs_str!("-Command"),
            crate::rs_str!(
                "Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty Name"
            ),
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let out_str = String::from_utf8_lossy(&output.stdout);
    let trimmed = out_str.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

#[cfg(target_os = "windows")]
fn get_bios_uuid() -> Option<String> {
    let output = Command::new(crate::rs_str!("powershell"))
        .args([
            crate::rs_str!("-NoProfile"),
            crate::rs_str!("-Command"),
            crate::rs_str!(
                "Get-CimInstance Win32_ComputerSystemProduct | Select-Object -ExpandProperty UUID"
            ),
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let out_str = String::from_utf8_lossy(&output.stdout);
    let trimmed = out_str.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

#[cfg(target_os = "windows")]
fn get_disk_serial() -> Option<String> {
    let output = Command::new(crate::rs_str!("powershell"))
        .args([crate::rs_str!("-NoProfile"), crate::rs_str!("-Command"), crate::rs_str!("Get-CimInstance Win32_DiskDrive | Select-Object -ExpandProperty SerialNumber | Select-Object -First 1")])
        .output().ok()?;
    if !output.status.success() {
        return None;
    }
    let out_str = String::from_utf8_lossy(&output.stdout);
    let trimmed = out_str.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

#[cfg(target_os = "macos")]
fn get_gpu_id() -> Option<String> {
    let output = Command::new(crate::rs_str!("system_profiler"))
        .args([crate::rs_str!("SPDisplaysDataType")])
        .output()
        .ok()?;
    let out_str = String::from_utf8_lossy(&output.stdout);
    out_str
        .lines()
        .find(|l| l.contains(&crate::rs_str!("Chipset Model:")))
        .map(|s| {
            s.replace(&crate::rs_str!("Chipset Model:"), "")
                .trim()
                .to_string()
        })
}

#[cfg(target_os = "macos")]
fn get_bios_uuid() -> Option<String> {
    let output = Command::new(crate::rs_str!("system_profiler"))
        .args([crate::rs_str!("SPHardwareDataType")])
        .output()
        .ok()?;
    let out_str = String::from_utf8_lossy(&output.stdout);
    out_str
        .lines()
        .find(|l| l.contains(&crate::rs_str!("Hardware UUID:")))
        .map(|s| {
            s.replace(&crate::rs_str!("Hardware UUID:"), "")
                .trim()
                .to_string()
        })
}

#[cfg(target_os = "macos")]
fn get_disk_serial() -> Option<String> {
    let output = Command::new(crate::rs_str!("system_profiler"))
        .args([crate::rs_str!("SPStorageDataType")])
        .output()
        .ok()?;
    let out_str = String::from_utf8_lossy(&output.stdout);
    out_str
        .lines()
        .find(|l| l.contains(&crate::rs_str!("Volume UUID:")))
        .map(|s| {
            s.replace(&crate::rs_str!("Volume UUID:"), "")
                .trim()
                .to_string()
        })
        .or_else(|| {
            out_str
                .lines()
                .find(|l| l.contains(&crate::rs_str!("Device Identifier:")))
                .map(|s| {
                    s.replace(&crate::rs_str!("Device Identifier:"), "")
                        .trim()
                        .to_string()
                })
        })
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn get_gpu_id() -> Option<String> {
    let output = Command::new(crate::rs_str!("lspci")).output().ok()?;
    let out_str = String::from_utf8_lossy(&output.stdout);
    let res = out_str
        .lines()
        .find(|l| {
            l.to_lowercase().contains(&crate::rs_str!("vga"))
                || l.to_lowercase().contains(&crate::rs_str!("3d"))
        })
        .map(|s| s.trim().to_string());
    println!("GET_GPU_ID: {:?}", res);
    res
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn get_bios_uuid() -> Option<String> {
    if let Ok(uuid) = std::fs::read_to_string(crate::rs_str!("/sys/class/dmi/id/product_uuid")) {
        let trimmed = uuid.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Ok(machine_id) = std::fs::read_to_string(crate::rs_str!("/etc/machine-id")) {
        let trimmed = machine_id.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn get_disk_serial() -> Option<String> {
    let output = Command::new(crate::rs_str!("lsblk"))
        .args([crate::rs_str!("-no"), crate::rs_str!("SERIAL")])
        .output()
        .ok()?;
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty())
        .map(|s| s.to_string())
}

//

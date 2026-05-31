#![allow(warnings)]

#[cfg(target_os = "linux")]
mod linux_impl {
    use libc::{kill, ptrace, PTRACE_ATTACH, PTRACE_DETACH, SIGCONT, SIGSTOP};
    use std::env;
    use std::fs::OpenOptions;
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::process;
    use std::thread;
    use std::time::Duration;
    use sysinfo::System;

    pub fn run() {
        let args: Vec<String> = env::args().collect();
        if args.len() < 3 || args[1] != "--attack" {
            eprintln!("Usage: rustshield-attacker --attack <ptrace|scan|inject|freeze>");
            process::exit(1);
        }

        let mode = args[2].as_str();

        println!("[*] Scanning for rustshield-mock-game...");
        let mut sys = System::new_all();
        sys.refresh_all();

        let target_pid = sys.processes().iter().find_map(|(&pid, p)| {
            if p.name().to_string_lossy().contains("rustshield-mock") {
                Some(pid.as_u32() as i32)
            } else {
                None
            }
        });

        let pid = match target_pid {
            Some(p) => p,
            None => {
                eprintln!("[-] rustshield-mock-game is not running!");
                process::exit(1);
            }
        };

        println!("[+] Found target PID: {}", pid);

        match mode {
            "ptrace" => attack_ptrace(pid),
            "scan" => attack_scan(pid),
            "inject" => attack_inject(pid),
            "freeze" => attack_freeze(pid),
            _ => {
                eprintln!("[-] Unknown attack mode.");
                process::exit(1);
            }
        }
    }

    fn attack_ptrace(pid: i32) {
        println!("[*] Initiating PTRACE_ATTACH attack (GDB/Debugger Simulator)...");
        unsafe {
            if ptrace(PTRACE_ATTACH, pid, std::ptr::null_mut::<libc::c_void>(), 0) == -1 {
                eprintln!("[-] PTRACE_ATTACH failed. Permission denied or already traced.");
                return;
            }
        }
        println!("[+] Attached successfully! Holding thread for 3 seconds...");
        thread::sleep(Duration::from_secs(3));

        println!("[*] Detaching...");
        unsafe {
            ptrace(PTRACE_DETACH, pid, std::ptr::null_mut::<libc::c_void>(), 0);
        }
        println!("[+] Attack complete.");
    }

    fn attack_scan(pid: i32) {
        println!("[*] Initiating Aggressive Memory Scan (Cheat Engine Simulator)...");
        let mem_path = format!("/proc/{}/mem", pid);

        match OpenOptions::new().read(true).open(&mem_path) {
            Ok(mut f) => {
                println!("[+] Opened {} for reading.", mem_path);
                let mut buffer = vec![0u8; 1024 * 1024 * 10]; // 10MB chunk
                let start_addr: u64 = 0x400000; // Typical Linux binary base

                if f.seek(SeekFrom::Start(start_addr)).is_ok() {
                    println!("[*] Dumping 10MB from 0x{:x}...", start_addr);
                    match f.read_exact(&mut buffer) {
                        Ok(_) => println!("[+] Read successful! Triggering page faults."),
                        Err(e) => eprintln!("[-] Read failed: {}", e),
                    }
                } else {
                    eprintln!("[-] Seek failed.");
                }
            }
            Err(e) => eprintln!(
                "[-] Failed to open memory: {}. (Tip: Need same user or ptrace rights)",
                e
            ),
        }
    }

    fn attack_inject(pid: i32) {
        println!("[*] Initiating Memory Injection (DLL/Shellcode Injector Simulator)...");
        let mem_path = format!("/proc/{}/mem", pid);

        match OpenOptions::new().write(true).open(&mem_path) {
            Ok(mut f) => {
                println!("[+] Opened {} for writing.", mem_path);
                let start_addr: u64 = 0x400000; // Risky arbitrary address

                if f.seek(SeekFrom::Start(start_addr)).is_ok() {
                    println!("[*] Injecting malicious payload at 0x{:x}...", start_addr);
                    let payload = vec![0x90; 1024]; // 1KB NOP sled
                    match f.write_all(&payload) {
                        Ok(_) => println!("[+] Injection successful!"),
                        Err(e) => eprintln!("[-] Injection failed (usually mapped read-only): {}", e),
                    }
                }
            }
            Err(e) => eprintln!("[-] Failed to open memory for writing: {}", e),
        }
    }

    fn attack_freeze(pid: i32) {
        println!("[*] Initiating Time Freeze (Thread Suspend Simulator)...");
        println!("[*] Sending SIGSTOP...");
        unsafe {
            if kill(pid, SIGSTOP) == 0 {
                println!("[+] SIGSTOP sent. Target is frozen.");
                println!("[*] Holding for 5 seconds to trigger Time Guard AI...");
                thread::sleep(Duration::from_secs(5));
                println!("[*] Sending SIGCONT...");
                kill(pid, SIGCONT);
                println!("[+] Target resumed.");
            } else {
                eprintln!("[-] Failed to send SIGSTOP.");
            }
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod stub {
    pub fn run() {
        println!("rustshield-attacker is only supported on Linux.");
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    linux_impl::run();

    #[cfg(not(target_os = "linux"))]
    stub::run();
}


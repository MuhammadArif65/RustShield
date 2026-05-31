#![allow(clippy::new_without_default)]
#![allow(dead_code, unused_variables, unused_imports)]
use crate::error::{FerrumWardError, Result};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use core::arch::x86_64::_rdtsc;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

pub static GLOBAL_AI: OnceLock<Mutex<NeuralHeuristicEngine>> = OnceLock::new();

/// A lightweight Zero-Dependency Neural Heuristic Engine
/// Evaluates runtime system behavior to detect debuggers, hypervisors, and memory scanners.
pub struct NeuralHeuristicEngine {
    // Neural Network Weights (Pre-trained Offline)
    // 5 Inputs -> 4 Hidden Nodes -> 1 Output
    hidden_weights: [[f32; 5]; 4],
    hidden_biases: [f32; 4],
    output_weights: [f32; 4],
    output_bias: f32,

    last_check: Instant,
    last_page_faults: u64,
}

impl NeuralHeuristicEngine {
    pub fn get_global() -> &'static Mutex<NeuralHeuristicEngine> {
        GLOBAL_AI.get_or_init(|| Mutex::new(NeuralHeuristicEngine::new()))
    }

    pub fn new() -> Self {
        // Pre-trained weights for the heuristic detection
        Self {
            hidden_weights: [
                [0.8, 0.4, 0.2, 0.9, 0.1], // Node 1 focuses on page faults and entropy
                [0.1, 0.9, 0.3, 0.2, 0.8], // Node 2 focuses on latency and decoy
                [0.5, 0.5, 0.5, 0.5, 0.5], // Node 3 balanced
                [0.9, 0.1, 0.8, 0.1, 0.2], // Node 4 focuses on time drift
            ],
            hidden_biases: [-0.5, -0.5, -1.0, -0.5],
            output_weights: [0.6, 0.7, 0.3, 0.8],
            output_bias: -0.2,
            last_check: Instant::now(),
            last_page_faults: 0,
        }
    }

    /// Measures CPU Instruction Latency using RDTSC (Detects Hypervisors / Debugger Step-Overs)
    fn measure_instruction_latency(&self) -> f32 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe {
            let start = _rdtsc();
            // Do a tiny bit of math to prevent instruction reordering
            std::hint::black_box(1 + 1);
            let end = _rdtsc();
            let delta = end.saturating_sub(start);
            // Normal execution takes ~20-100 cycles. A hypervisor VM-exit can take 1000+ cycles.
            if delta > 1000 {
                1.0
            } else {
                (delta as f32) / 1000.0
            }
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            0.0
        }
    }

    /// Estimates page faults by reading /proc/self/stat on Linux.
    /// A sudden massive spike indicates a memory scanner like Cheat Engine.
    fn measure_page_faults(&mut self) -> f32 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(stat) = std::fs::read_to_string("/proc/self/stat") {
                let parts: Vec<&str> = stat.split_whitespace().collect();
                if parts.len() > 11 {
                    if let Ok(minflt) = parts[9].parse::<u64>() {
                        if let Ok(majflt) = parts[11].parse::<u64>() {
                            let total = minflt + majflt;
                            let delta = total.saturating_sub(self.last_page_faults);
                            self.last_page_faults = total;

                            // If delta > 1000 in a short time, it's very suspicious
                            return if delta > 1000 {
                                1.0
                            } else {
                                (delta as f32) / 1000.0
                            };
                        }
                    }
                }
            }
        }
        0.0 // Fallback for Windows/macOS where we don't pull heavy dependencies for this example
    }

    /// Measures Time Drift Variance
    fn measure_time_drift(&self) -> f32 {
        let elapsed = self.last_check.elapsed().as_millis() as f32;
        // If the check loop took significantly longer than expected (e.g. paused in debugger)
        // Note: The caller should call this roughly every 50-300ms.
        if elapsed > 1000.0 {
            1.0
        } else {
            elapsed / 1000.0
        }
    }

    /// Measures Memory Entropy of the code segment to detect injected shellcode
    fn measure_memory_entropy(&self) -> f32 {
        // In a real scenario, we'd sample a random chunk of our own executable memory.
        // For zero-dependency, we simulate a fast pseudo-random sampling.
        let mut entropy_score = 0.0;
        let ptr = NeuralHeuristicEngine::measure_time_drift as *const () as *const u8;
        let mut byte_counts = [0usize; 256];
        unsafe {
            for i in 0..1024 {
                // Safety: Reading our own .text segment is generally safe and mapped
                // We use a small bounded read.
                let val = ptr.add(i).read_volatile();
                byte_counts[val as usize] += 1;
            }
        }

        let mut entropy = 0.0_f32;
        for &count in byte_counts.iter() {
            if count > 0 {
                let p = (count as f32) / 1024.0;
                entropy -= p * p.log2();
            }
        }
        // Max entropy for 256 bytes is 8.0. Normal code is around 4.0 - 6.0.
        // If it's near 8.0, it's heavily compressed or encrypted shellcode.
        if entropy > 7.5 {
            entropy_score = 1.0;
        } else if entropy > 6.5 {
            entropy_score = 0.5;
        }
        entropy_score
    }

    /// Profiles decoy accesses by checking if the honeypot has been tampered with.
    fn measure_decoy_profiling(&self) -> f32 {
        // Check if the global honeypot has been triggered by a memory scanner
        if let Some(state) = crate::protection::integrity::get_active_state() {
            if state.honeypot.verify().is_err() {
                return 1.0; // Honeypot was tampered — maximum suspicion
            }
        }
        0.0
    }

    fn sigmoid(x: f32) -> f32 {
        1.0 / (1.0 + (-x).exp())
    }

    /// Executes the forward pass of the Neural Network
    pub fn evaluate(&mut self) -> Result<f32> {
        let features = [
            self.measure_page_faults(),
            self.measure_instruction_latency(),
            self.measure_time_drift(),
            self.measure_memory_entropy(),
            self.measure_decoy_profiling(),
        ];

        self.last_check = Instant::now();

        let mut hidden_layer = [0.0; 4];
        for (i, node) in hidden_layer.iter_mut().enumerate() {
            let mut sum = self.hidden_biases[i];
            for (j, &feature) in features.iter().enumerate() {
                sum += feature * self.hidden_weights[i][j];
            }
            *node = Self::sigmoid(sum);
        }

        let mut output_sum = self.output_bias;
        for (i, &hidden_val) in hidden_layer.iter().enumerate() {
            output_sum += hidden_val * self.output_weights[i];
        }

        let final_score = Self::sigmoid(output_sum);

        if final_score > 0.85 {
            Err(FerrumWardError::TamperDetected)
        } else {
            Ok(final_score)
        }
    }
}

//

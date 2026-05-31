use crate::error::{FerrumWardError, Result};
use std::sync::Mutex;

/// Tracks kinematic inputs (like mouse click intervals) to detect macros and auto-clickers.
/// An auto-clicker typically clicks at perfectly mathematically uniform intervals (e.g. exactly 50ms),
/// which is impossible for a human. If the standard deviation of the timing approaches zero,
/// we flag it as an anomaly.
pub struct KinematicAnomalyDetector {
    click_intervals: Mutex<Vec<u64>>,
    max_samples: usize,
}

impl Default for KinematicAnomalyDetector {
    fn default() -> Self {
        Self::new(20) // Keep last 20 click intervals
    }
}

impl KinematicAnomalyDetector {
    pub fn new(max_samples: usize) -> Self {
        Self {
            click_intervals: Mutex::new(Vec::with_capacity(max_samples)),
            max_samples,
        }
    }

    /// Feeds a new click interval (in milliseconds) to the AI detector.
    /// Returns TamperDetected if the variance is inhumanly perfect.
    pub fn feed_click_interval(&self, delta_ms: u64) -> Result<()> {
        let mut intervals = self
            .click_intervals
            .lock()
            .map_err(|_| FerrumWardError::TamperDetected)?;

        if intervals.len() >= self.max_samples {
            intervals.remove(0);
        }
        intervals.push(delta_ms);

        if intervals.len() == self.max_samples {
            // Calculate mean
            let sum: u64 = intervals.iter().sum();
            let mean = sum as f64 / self.max_samples as f64;

            // Calculate variance
            let mut variance_sum = 0.0;
            for &val in intervals.iter() {
                let diff = val as f64 - mean;
                variance_sum += diff * diff;
            }
            let variance = variance_sum / self.max_samples as f64;

            // Calculate standard deviation
            let std_dev = variance.sqrt();

            // If the std_dev is < 1.0ms over 20 samples, it's 100% a robot/macro.
            // We removed the `mean > 10.0` check as ultra-fast macros were bypassing it.
            if std_dev < 1.0 {
                // We clear the buffer so we don't spam errors
                intervals.clear();
                return Err(FerrumWardError::TamperDetected);
            }
        }

        Ok(())
    }
}

//

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttemptMetrics {
    pub accepted: bool,
    pub duration: Duration,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PerformanceMetrics {
    pub total_elapsed: Duration,
    pub parse_elapsed: Duration,
    pub render_elapsed: Duration,
    pub check_elapsed: Duration,
    pub algorithm_elapsed: Duration,
    pub attempt_elapsed: Duration,
    pub attempt_count: usize,
    pub accepted_attempts: usize,
    pub rejected_attempts: usize,
}

impl PerformanceMetrics {
    pub fn record_attempt(&mut self, attempt: AttemptMetrics) {
        self.attempt_count += 1;
        self.attempt_elapsed += attempt.duration;
        if attempt.accepted {
            self.accepted_attempts += 1;
        } else {
            self.rejected_attempts += 1;
        }
    }
}

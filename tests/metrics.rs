use std::time::Duration;

use svdd::metrics::{AttemptMetrics, PerformanceMetrics};

#[test]
fn aggregates_attempt_and_phase_durations() {
    let mut metrics = PerformanceMetrics::default();
    metrics.parse_elapsed = Duration::from_millis(3);
    metrics.render_elapsed = Duration::from_millis(5);
    metrics.check_elapsed = Duration::from_millis(7);
    metrics.algorithm_elapsed = Duration::from_millis(11);
    metrics.total_elapsed = Duration::from_millis(29);

    metrics.record_attempt(AttemptMetrics {
        accepted: true,
        duration: Duration::from_millis(13),
    });
    metrics.record_attempt(AttemptMetrics {
        accepted: false,
        duration: Duration::from_millis(17),
    });

    assert_eq!(metrics.attempt_count, 2);
    assert_eq!(metrics.accepted_attempts, 1);
    assert_eq!(metrics.rejected_attempts, 1);
    assert_eq!(metrics.attempt_elapsed, Duration::from_millis(30));
    assert_eq!(metrics.total_elapsed, Duration::from_millis(29));
}

use std::collections::BTreeSet;

use svdd::algorithms::{naive::NaiveReducer, ReductionAlgorithm};
use svdd::check::CheckOutcome;
use svdd::model::{CandidateKind, ReductionCandidate};
use svdd::session::{ReductionSession, SessionInput};

#[test]
fn naive_reducer_accepts_candidates_that_keep_property() {
    let candidates = vec![
        ReductionCandidate::new(0, CandidateKind::Node, 0, 1),
        ReductionCandidate::new(1, CandidateKind::Node, 1, 9),
    ];
    let session = ReductionSession::new(
        SessionInput::new("Xkeepdrop".into(), candidates),
        |rendered: &str, disabled: &BTreeSet<usize>| {
            if rendered.contains('X') || disabled.contains(&1) {
                CheckOutcome::Lost
            } else {
                CheckOutcome::Kept
            }
        },
    );

    let summary = NaiveReducer::default().run(session).unwrap();

    assert!(summary.disabled_candidates.contains(&0));
    assert!(!summary.disabled_candidates.contains(&1));
    assert_eq!(summary.metrics.attempt_count, 2);
    assert_eq!(summary.metrics.accepted_attempts, 1);
}

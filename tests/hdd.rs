use std::collections::BTreeSet;

use svdd::algorithms::{hdd::HddReducer, ReductionAlgorithm};
use svdd::check::CheckOutcome;
use svdd::model::{CandidateKind, ReductionCandidate};
use svdd::session::{ReductionSession, SessionInput};

#[test]
fn hdd_reducer_uses_hierarchy_and_records_metrics() {
    let candidates = vec![
        ReductionCandidate::with_parent(0, CandidateKind::Node, 0, 2, None),
        ReductionCandidate::with_parent(1, CandidateKind::Node, 2, 4, Some(0)),
        ReductionCandidate::with_parent(2, CandidateKind::Node, 4, 6, Some(0)),
    ];
    let session = ReductionSession::new(
        SessionInput::new("abcdef".into(), candidates),
        |_rendered: &str, disabled: &BTreeSet<usize>| {
            if disabled.contains(&1) && disabled.contains(&2) {
                CheckOutcome::Kept
            } else {
                CheckOutcome::Lost
            }
        },
    );

    let summary = HddReducer::default().run(session).unwrap();

    assert!(summary.disabled_candidates.contains(&1));
    assert!(summary.disabled_candidates.contains(&2));
    assert!(summary.metrics.attempt_count >= 1);
}

#[test]
fn hdd_can_chunk_top_level_siblings() {
    let candidates = vec![
        ReductionCandidate::new(0, CandidateKind::Node, 0, 2),
        ReductionCandidate::new(1, CandidateKind::Node, 2, 4),
    ];
    let session = ReductionSession::new(
        SessionInput::new("abcdef".into(), candidates),
        |_rendered: &str, disabled: &BTreeSet<usize>| {
            if disabled.contains(&0) && disabled.contains(&1) {
                CheckOutcome::Kept
            } else {
                CheckOutcome::Lost
            }
        },
    );

    let summary = HddReducer::default().run(session).unwrap();

    assert!(summary.disabled_candidates.contains(&0));
    assert!(summary.disabled_candidates.contains(&1));
}

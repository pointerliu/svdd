use std::collections::BTreeSet;

use svdd::algorithms::{ddmin::DdminReducer, ReductionAlgorithm};
use svdd::check::CheckOutcome;
use svdd::model::{CandidateKind, ReductionCandidate};
use svdd::session::{ReductionSession, SessionInput};

#[test]
fn ddmin_reduces_to_minimal_failure_subset() {
    let candidates = vec![
        ReductionCandidate::new(0, CandidateKind::Node, 0, 1),
        ReductionCandidate::new(1, CandidateKind::Node, 1, 2),
        ReductionCandidate::new(2, CandidateKind::Node, 2, 3),
        ReductionCandidate::new(3, CandidateKind::Node, 3, 4),
    ];
    let session = ReductionSession::new(
        SessionInput::new("abcd".into(), candidates),
        |_rendered: &str, disabled: &BTreeSet<usize>| {
            if disabled.contains(&0)
                && disabled.contains(&1)
                && !disabled.contains(&2)
                && !disabled.contains(&3)
            {
                CheckOutcome::Kept
            } else {
                CheckOutcome::Lost
            }
        },
    );

    let summary = DdminReducer::default().run(session).unwrap();

    assert!(summary.disabled_candidates.contains(&0));
    assert!(summary.disabled_candidates.contains(&1));
    assert!(!summary.disabled_candidates.contains(&2));
    assert!(!summary.disabled_candidates.contains(&3));
}

#[test]
fn ddmin_uses_structural_groups_before_single_candidates() {
    let candidates = vec![
        ReductionCandidate::new(0, CandidateKind::Node, 0, 1),
        ReductionCandidate::new(1, CandidateKind::Statement, 1, 2),
        ReductionCandidate::new(2, CandidateKind::Node, 2, 3),
    ];
    let session = ReductionSession::new(
        SessionInput::new("abc".into(), candidates),
        |_rendered: &str, disabled: &BTreeSet<usize>| {
            if disabled.contains(&0) && disabled.contains(&2) && !disabled.contains(&1) {
                CheckOutcome::Kept
            } else {
                CheckOutcome::Lost
            }
        },
    );

    let summary = DdminReducer::default().run(session).unwrap();

    assert_eq!(summary.disabled_candidates, BTreeSet::from([0, 2]));
    assert_eq!(summary.metrics.attempt_count, 2);
}

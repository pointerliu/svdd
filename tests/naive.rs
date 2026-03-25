use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

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
    assert_eq!(summary.metrics.attempt_count, 3);
    assert_eq!(summary.metrics.accepted_attempts, 1);
}

#[test]
fn naive_retries_candidates_until_fixpoint() {
    let candidates = vec![
        ReductionCandidate::new(0, CandidateKind::Node, 0, 1),
        ReductionCandidate::new(1, CandidateKind::Node, 1, 2),
    ];
    let session = ReductionSession::new(
        SessionInput::new("ab".into(), candidates),
        |_rendered: &str, disabled: &BTreeSet<usize>| {
            if disabled.contains(&1) {
                CheckOutcome::Kept
            } else if disabled.contains(&0) {
                CheckOutcome::Lost
            } else {
                CheckOutcome::Lost
            }
        },
    );

    let summary = NaiveReducer::default().run(session).unwrap();

    assert!(summary.disabled_candidates.contains(&1));
    assert!(summary.disabled_candidates.contains(&0));
}

#[test]
fn naive_prefers_deeper_candidates_first() {
    let mut parent = ReductionCandidate::new(0, CandidateKind::Node, 0, 1);
    parent.depth = 0;
    let mut child = ReductionCandidate::new(1, CandidateKind::Node, 1, 2);
    child.depth = 1;

    let attempts = Arc::new(Mutex::new(Vec::new()));
    let attempts_ref = attempts.clone();
    let session = ReductionSession::new(
        SessionInput::new("ab".into(), vec![parent, child]),
        move |_rendered: &str, disabled: &BTreeSet<usize>| {
            attempts_ref.lock().unwrap().push(disabled.clone());
            CheckOutcome::Lost
        },
    );

    let _summary = NaiveReducer::default().run(session).unwrap();
    let attempts = attempts.lock().unwrap();

    assert_eq!(attempts[0], BTreeSet::from([1]));
    assert_eq!(attempts[1], BTreeSet::from([0]));
}

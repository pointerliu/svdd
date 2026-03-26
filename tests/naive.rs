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
fn naive_uses_depth_as_tiebreaker_for_same_size_candidates() {
    let mut parent = ReductionCandidate::new(0, CandidateKind::Node, 0, 1);
    parent.depth = 0;
    parent.line_count = 1;
    let mut child = ReductionCandidate::new(1, CandidateKind::Node, 1, 2);
    child.depth = 1;
    child.line_count = 1;

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

#[test]
fn naive_prefers_larger_candidates_before_deeper_ones() {
    let mut large_parent = ReductionCandidate::new(0, CandidateKind::Node, 0, 8);
    large_parent.depth = 0;
    large_parent.line_count = 4;

    let mut small_child = ReductionCandidate::new(1, CandidateKind::Node, 2, 3);
    small_child.depth = 1;
    small_child.parent_id = Some(0);
    small_child.line_count = 1;

    let attempts = Arc::new(Mutex::new(Vec::new()));
    let attempts_ref = attempts.clone();
    let session = ReductionSession::new(
        SessionInput::new("abcdefgh".into(), vec![large_parent, small_child]),
        move |_rendered: &str, disabled: &BTreeSet<usize>| {
            attempts_ref.lock().unwrap().push(disabled.clone());
            CheckOutcome::Lost
        },
    );

    let _summary = NaiveReducer::default().run(session).unwrap();
    let attempts = attempts.lock().unwrap();

    assert_eq!(attempts[0], BTreeSet::from([0]));
    assert_eq!(attempts[1], BTreeSet::from([1]));
}

#[test]
fn naive_skips_candidates_shadowed_by_disabled_ancestors() {
    let mut parent = ReductionCandidate::new(0, CandidateKind::Node, 0, 8);
    parent.line_count = 4;

    let mut child = ReductionCandidate::new(1, CandidateKind::Node, 2, 3);
    child.parent_id = Some(0);
    child.depth = 1;
    child.line_count = 1;

    let attempts = Arc::new(Mutex::new(Vec::new()));
    let attempts_ref = attempts.clone();
    let session = ReductionSession::new(
        SessionInput::new("abcdefgh".into(), vec![parent, child]),
        move |_rendered: &str, disabled: &BTreeSet<usize>| {
            attempts_ref.lock().unwrap().push(disabled.clone());
            if disabled.contains(&0) {
                CheckOutcome::Kept
            } else {
                CheckOutcome::Lost
            }
        },
    );

    let summary = NaiveReducer::default().run(session).unwrap();
    let attempts = attempts.lock().unwrap();

    assert_eq!(summary.disabled_candidates, BTreeSet::from([0]));
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0], BTreeSet::from([0]));
}

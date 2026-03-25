use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use svdd::check::CheckOutcome;
use svdd::model::{CandidateKind, ReductionCandidate};
use svdd::session::{ReductionSession, SessionInput};

#[test]
fn caches_duplicate_attempt_results() {
    let calls = Arc::new(Mutex::new(0usize));
    let calls_ref = calls.clone();
    let candidates = vec![ReductionCandidate::new(0, CandidateKind::Node, 0, 1)];
    let mut session = ReductionSession::new(
        SessionInput::new("x".into(), candidates),
        move |_rendered: &str, _disabled: &BTreeSet<usize>| {
            *calls_ref.lock().unwrap() += 1;
            CheckOutcome::Lost
        },
    );

    let mut disabled = BTreeSet::new();
    assert!(!session.attempt_disable(&mut disabled, &[0]).unwrap());
    assert!(!session.attempt_disable(&mut disabled, &[0]).unwrap());

    assert_eq!(*calls.lock().unwrap(), 1);
}

#[test]
fn caches_equivalent_rendered_attempts() {
    let calls = Arc::new(Mutex::new(0usize));
    let calls_ref = calls.clone();
    let mut parent = ReductionCandidate::new(0, CandidateKind::Node, 0, 3);
    parent.children.push(1);
    let mut child = ReductionCandidate::new(1, CandidateKind::Node, 1, 2);
    child.parent_id = Some(0);
    child.depth = 1;

    let mut session = ReductionSession::new(
        SessionInput::new("abc".into(), vec![parent, child]),
        move |_rendered: &str, _disabled: &BTreeSet<usize>| {
            *calls_ref.lock().unwrap() += 1;
            CheckOutcome::Lost
        },
    );

    let mut disabled = BTreeSet::new();
    assert!(!session.attempt_disable(&mut disabled, &[0]).unwrap());
    disabled.insert(0);
    assert!(!session.attempt_disable(&mut disabled, &[1]).unwrap());

    assert_eq!(*calls.lock().unwrap(), 1);
}

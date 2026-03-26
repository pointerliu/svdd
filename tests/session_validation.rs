use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use svdd::check::CheckOutcome;
use svdd::cli::SyntaxCheckMode;
use svdd::model::{CandidateKind, ReductionCandidate};
use svdd::session::{ReductionSession, SessionInput};

#[test]
fn invalid_intermediate_source_is_rejected_before_checker_runs() {
    let calls = Arc::new(Mutex::new(0usize));
    let checker_calls = calls.clone();
    let candidates = vec![ReductionCandidate::new(0, CandidateKind::Node, 0, 7)];
    let mut session = ReductionSession::new(
        SessionInput::new("module top; endmodule\n".into(), candidates),
        move |_rendered: &str, _disabled: &BTreeSet<usize>| {
            *checker_calls.lock().unwrap() += 1;
            CheckOutcome::Kept
        },
    )
    .with_parse_validation("trial.sv", SyntaxCheckMode::Always);

    let mut disabled = BTreeSet::new();
    let accepted = session.attempt_disable(&mut disabled, &[0]).unwrap();

    assert!(!accepted);
    assert_eq!(*calls.lock().unwrap(), 0);
}

#[test]
fn invalid_intermediate_source_reaches_checker_when_syntax_check_is_off() {
    let calls = Arc::new(Mutex::new(0usize));
    let checker_calls = calls.clone();
    let candidates = vec![ReductionCandidate::new(0, CandidateKind::Node, 0, 7)];
    let mut session = ReductionSession::new(
        SessionInput::new("module top; endmodule\n".into(), candidates),
        move |_rendered: &str, _disabled: &BTreeSet<usize>| {
            *checker_calls.lock().unwrap() += 1;
            CheckOutcome::Lost
        },
    )
    .with_parse_validation("trial.sv", SyntaxCheckMode::Off);

    let mut disabled = BTreeSet::new();
    let accepted = session.attempt_disable(&mut disabled, &[0]).unwrap();

    assert!(!accepted);
    assert_eq!(*calls.lock().unwrap(), 1);
}

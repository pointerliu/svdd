use std::collections::BTreeSet;

use tempfile::tempdir;

use svdd::check::CheckOutcome;
use svdd::model::{CandidateKind, ReductionCandidate};
use svdd::session::{ReductionSession, SessionInput};

#[test]
fn session_persists_attempt_files_in_output_dir() {
    let dir = tempdir().unwrap();
    let candidates = vec![ReductionCandidate::new(0, CandidateKind::Node, 0, 4)];
    let mut session = ReductionSession::new(
        SessionInput::new("dropkeep".into(), candidates),
        |_rendered: &str, _disabled: &BTreeSet<usize>| CheckOutcome::Kept,
    )
    .with_output_dir(dir.path().to_path_buf(), "input.sv");

    let mut disabled = BTreeSet::new();
    let accepted = session.attempt_disable(&mut disabled, &[0]).unwrap();

    assert!(accepted);
    assert!(dir.path().join("attempts/attempt-000001/input.sv").exists());
}

use std::collections::BTreeSet;

use svdd::algorithms::{hddmin::HddminReducer, ReductionAlgorithm};
use svdd::check::CheckOutcome;
use svdd::model::{CandidateKind, ReductionCandidate};
use svdd::session::{ReductionSession, SessionInput};

#[test]
fn hddmin_reduces_one_level_at_a_time() {
    let mut keep_parent = ReductionCandidate::new(0, CandidateKind::Node, 0, 1);
    keep_parent.depth = 0;
    keep_parent.children = vec![2, 3];

    let mut drop_parent = ReductionCandidate::new(1, CandidateKind::Node, 1, 2);
    drop_parent.depth = 0;
    drop_parent.children = vec![4];

    let mut keep_child = ReductionCandidate::new(2, CandidateKind::Node, 2, 3);
    keep_child.depth = 1;
    keep_child.parent_id = Some(0);

    let mut drop_child = ReductionCandidate::new(3, CandidateKind::Node, 3, 4);
    drop_child.depth = 1;
    drop_child.parent_id = Some(0);

    let mut hidden_child = ReductionCandidate::new(4, CandidateKind::Node, 4, 5);
    hidden_child.depth = 1;
    hidden_child.parent_id = Some(1);

    let session = ReductionSession::new(
        SessionInput::new(
            "abcde".into(),
            vec![
                keep_parent,
                drop_parent,
                keep_child,
                drop_child,
                hidden_child,
            ],
        ),
        |_rendered: &str, disabled: &BTreeSet<usize>| {
            if !disabled.contains(&0) && !disabled.contains(&2) {
                CheckOutcome::Kept
            } else {
                CheckOutcome::Lost
            }
        },
    );

    let summary = HddminReducer::default().run(session).unwrap();

    assert!(!summary.disabled_candidates.contains(&0));
    assert!(summary.disabled_candidates.contains(&1));
    assert!(!summary.disabled_candidates.contains(&2));
    assert!(summary.disabled_candidates.contains(&3));
}

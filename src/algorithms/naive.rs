use std::collections::BTreeSet;
use std::time::Instant;

use anyhow::Result;

use crate::algorithms::ReductionAlgorithm;
use crate::model::ReductionSummary;
use crate::session::ReductionSession;

#[derive(Debug, Default, Clone, Copy)]
pub struct NaiveReducer;

impl ReductionAlgorithm for NaiveReducer {
    fn name(&self) -> &'static str {
        "naive"
    }

    fn run(&self, mut session: ReductionSession) -> Result<ReductionSummary> {
        let total_start = Instant::now();
        let algo_start = Instant::now();
        let mut disabled = BTreeSet::new();
        let candidate_ids: Vec<usize> = session.candidate_ids().collect();
        loop {
            let mut changed = false;
            for id in &candidate_ids {
                if disabled.contains(id) {
                    continue;
                }
                changed |= session.attempt_disable(&mut disabled, &[*id])?;
            }
            if !changed {
                break;
            }
        }
        session.metrics_mut().algorithm_elapsed += algo_start.elapsed();
        Ok(session.finalize(disabled, total_start.elapsed()))
    }
}

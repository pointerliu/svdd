use std::collections::BTreeSet;
use std::time::Instant;

use anyhow::Result;

use crate::algorithms::ddmin::ddmin;
use crate::algorithms::ReductionAlgorithm;
use crate::model::ReductionSummary;
use crate::session::ReductionSession;

#[derive(Debug, Default, Clone, Copy)]
pub struct HddminReducer;

impl ReductionAlgorithm for HddminReducer {
    fn name(&self) -> &'static str {
        "hddmin"
    }

    fn run(&self, mut session: ReductionSession) -> Result<ReductionSummary> {
        let total_start = Instant::now();
        let algo_start = Instant::now();
        let mut disabled = BTreeSet::new();

        for depth in session.depths() {
            let level_ids = session.level_candidate_ids(depth, &disabled);
            if level_ids.is_empty() {
                continue;
            }

            let level_set: BTreeSet<usize> = level_ids.iter().copied().collect();
            let kept = ddmin(level_ids.clone(), |subset| {
                let kept: BTreeSet<usize> = subset.into_iter().collect();
                let to_disable: Vec<usize> = level_ids
                    .iter()
                    .copied()
                    .filter(|id| !kept.contains(id))
                    .collect();
                let mut trial_disabled = disabled.clone();
                session.attempt_disable(&mut trial_disabled, &to_disable)
            })?;

            for id in level_set {
                if !kept.contains(&id) {
                    disabled.insert(id);
                }
            }
        }

        session.metrics_mut().algorithm_elapsed += algo_start.elapsed();
        Ok(session.finalize(disabled, total_start.elapsed()))
    }
}

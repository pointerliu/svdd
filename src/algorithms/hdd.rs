use std::collections::BTreeSet;
use std::time::Instant;

use anyhow::Result;

use crate::algorithms::ReductionAlgorithm;
use crate::model::ReductionSummary;
use crate::session::ReductionSession;

#[derive(Debug, Default, Clone, Copy)]
pub struct HddReducer;

impl ReductionAlgorithm for HddReducer {
    fn name(&self) -> &'static str {
        "hdd"
    }

    fn run(&self, mut session: ReductionSession) -> Result<ReductionSummary> {
        let total_start = Instant::now();
        let algo_start = Instant::now();
        let mut disabled = BTreeSet::new();

        loop {
            let mut changed = false;

            for group in session.grouped_siblings() {
                changed |= reduce_group(&mut session, &mut disabled, &group)?;
            }

            let remaining_ids: Vec<usize> = session.candidate_ids().collect();
            for id in remaining_ids {
                if disabled.contains(&id) {
                    continue;
                }
                changed |= session.attempt_disable(&mut disabled, &[id])?;
            }

            if !changed {
                break;
            }
        }

        session.metrics_mut().algorithm_elapsed += algo_start.elapsed();
        Ok(session.finalize(disabled, total_start.elapsed()))
    }
}

fn reduce_group(
    session: &mut ReductionSession,
    disabled: &mut BTreeSet<usize>,
    group: &[usize],
) -> Result<bool> {
    let mut active: Vec<usize> = group
        .iter()
        .copied()
        .filter(|id| !disabled.contains(id))
        .collect();
    if active.is_empty() {
        return Ok(false);
    }

    let mut changed = false;

    if active.len() > 1 && session.attempt_disable(disabled, &active)? {
        return Ok(true);
    }

    let mut granularity = 2usize.min(active.len().max(1));
    while active.len() > 1 {
        let chunks = partition(&active, granularity);
        let mut accepted = false;

        for chunk in chunks {
            if session.attempt_disable(disabled, &chunk)? {
                let removed: BTreeSet<usize> = chunk.into_iter().collect();
                active.retain(|id| !removed.contains(id));
                granularity = 2usize.min(active.len().max(1));
                accepted = true;
                changed = true;
                break;
            }
        }

        if !accepted {
            if granularity >= active.len() {
                break;
            }
            granularity = (granularity * 2).min(active.len());
        }
    }

    for id in active {
        if disabled.contains(&id) {
            continue;
        }
        changed |= session.attempt_disable(disabled, &[id])?;
    }

    Ok(changed)
}

fn partition(ids: &[usize], granularity: usize) -> Vec<Vec<usize>> {
    let chunk_size = ids.len().div_ceil(granularity).max(1);
    ids.chunks(chunk_size).map(|chunk| chunk.to_vec()).collect()
}

use std::collections::BTreeSet;
use std::time::Instant;

use anyhow::Result;

use crate::algorithms::ReductionAlgorithm;
use crate::model::ReductionSummary;
use crate::session::ReductionSession;

#[derive(Debug, Default, Clone, Copy)]
pub struct DdminReducer;

impl ReductionAlgorithm for DdminReducer {
    fn name(&self) -> &'static str {
        "ddmin"
    }

    fn run(&self, mut session: ReductionSession) -> Result<ReductionSummary> {
        let total_start = Instant::now();
        let algo_start = Instant::now();
        let mut disabled = BTreeSet::new();
        let candidates: Vec<usize> = session.candidate_ids().collect();
        let kept = ddmin(candidates, |subset| {
            let subset: BTreeSet<usize> = subset.into_iter().collect();
            let to_disable = complement_of(
                &subset,
                &disabled,
                &session.candidate_ids().collect::<Vec<_>>(),
            );
            let mut trial_disabled = disabled.clone();
            session.attempt_disable(&mut trial_disabled, &to_disable)
        })?;

        let final_ids: Vec<usize> = session.candidate_ids().collect();
        for id in final_ids {
            if !kept.contains(&id) {
                disabled.insert(id);
            }
        }

        session.metrics_mut().algorithm_elapsed += algo_start.elapsed();
        Ok(session.finalize(disabled, total_start.elapsed()))
    }
}

pub fn ddmin<F>(mut config: Vec<usize>, mut test: F) -> Result<BTreeSet<usize>>
where
    F: FnMut(Vec<usize>) -> Result<bool>,
{
    if config.len() <= 1 {
        return Ok(config.into_iter().collect());
    }

    let mut n = 2usize;
    while config.len() >= 2 {
        let subsets = partition(&config, n);
        let mut reduced = false;

        for subset in &subsets {
            if test(subset.clone())? {
                config = subset.clone();
                n = n.saturating_sub(1).max(2);
                reduced = true;
                break;
            }
        }
        if reduced {
            continue;
        }

        for subset in &subsets {
            let complement = difference(&config, subset);
            if complement.is_empty() {
                continue;
            }
            if test(complement.clone())? {
                config = complement;
                n = n.saturating_sub(1).max(2);
                reduced = true;
                break;
            }
        }
        if reduced {
            continue;
        }

        if n >= config.len() {
            break;
        }
        n = (n * 2).min(config.len());
    }

    Ok(config.into_iter().collect())
}

fn partition(config: &[usize], n: usize) -> Vec<Vec<usize>> {
    let chunk_size = config.len().div_ceil(n).max(1);
    config
        .chunks(chunk_size)
        .map(|chunk| chunk.to_vec())
        .collect()
}

fn difference(config: &[usize], subset: &[usize]) -> Vec<usize> {
    let subset: BTreeSet<usize> = subset.iter().copied().collect();
    config
        .iter()
        .copied()
        .filter(|id| !subset.contains(id))
        .collect()
}

fn complement_of(
    subset: &BTreeSet<usize>,
    already_disabled: &BTreeSet<usize>,
    universe: &[usize],
) -> Vec<usize> {
    universe
        .iter()
        .copied()
        .filter(|id| !subset.contains(id) && !already_disabled.contains(id))
        .collect()
}

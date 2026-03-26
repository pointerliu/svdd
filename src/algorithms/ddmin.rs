use std::collections::BTreeSet;
use std::time::Instant;

use anyhow::Result;

use crate::algorithms::ReductionAlgorithm;
use crate::model::ReductionSummary;
use crate::profile;
use crate::session::ReductionSession;

#[derive(Debug, Default, Clone, Copy)]
pub struct DdminReducer;

impl ReductionAlgorithm for DdminReducer {
    fn name(&self) -> &'static str {
        "ddmin"
    }

    fn run(&self, mut session: ReductionSession) -> Result<ReductionSummary> {
        let _scope = profile::Scope::new("algorithms::DdminReducer::run");
        let total_start = Instant::now();
        let algo_start = Instant::now();
        let mut disabled = BTreeSet::new();
        let groups = session.candidate_groups(&disabled);
        let group_ids: Vec<usize> = (0..groups.len()).collect();
        let kept_groups = ddmin(group_ids, |subset| {
            let subset: BTreeSet<usize> = subset.into_iter().collect();
            let to_disable = complement_groups(&subset, &groups);
            let mut trial_disabled = disabled.clone();
            session.attempt_disable(&mut trial_disabled, &to_disable)
        })?;

        for (group_id, group) in groups.iter().enumerate() {
            if !kept_groups.contains(&group_id) {
                disabled.extend(group.iter().copied());
            }
        }

        for group_id in kept_groups {
            let group = groups[group_id].clone();
            if group.len() <= 1 {
                continue;
            }

            let kept = ddmin(group.clone(), |subset| {
                let subset: BTreeSet<usize> = subset.into_iter().collect();
                let to_disable: Vec<usize> = group
                    .iter()
                    .copied()
                    .filter(|id| !subset.contains(id))
                    .collect();
                let mut trial_disabled = disabled.clone();
                session.attempt_disable(&mut trial_disabled, &to_disable)
            })?;

            for id in group {
                if !kept.contains(&id) {
                    disabled.insert(id);
                }
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
    let _scope = profile::Scope::new("algorithms::ddmin::ddmin");
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

fn complement_groups(subset: &BTreeSet<usize>, groups: &[Vec<usize>]) -> Vec<usize> {
    groups
        .iter()
        .enumerate()
        .filter(|(group_id, _)| !subset.contains(group_id))
        .flat_map(|(_, group)| group.iter().copied())
        .collect()
}

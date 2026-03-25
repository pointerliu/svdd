use std::collections::BTreeSet;

use anyhow::{anyhow, Result};

use crate::model::ReductionCandidate;

pub fn render_source(
    source: &str,
    candidates: &[ReductionCandidate],
    disabled: &BTreeSet<usize>,
) -> Result<String> {
    let mut spans = Vec::new();
    for id in disabled {
        let candidate = candidates
            .get(*id)
            .ok_or_else(|| anyhow!("unknown candidate id {id}"))?;
        if is_shadowed(candidate, candidates, disabled) {
            continue;
        }
        spans.push(candidate.span());
    }

    spans.sort_unstable_by_key(|span| span.0);
    let merged = merge_spans(spans);

    let mut rendered = String::with_capacity(source.len());
    let mut cursor = 0usize;
    for (start, end) in merged {
        if start > source.len() || end > source.len() || start > end {
            return Err(anyhow!(
                "candidate span {start}..{end} is outside the source"
            ));
        }
        rendered.push_str(&source[cursor..start]);
        cursor = end;
    }
    rendered.push_str(&source[cursor..]);

    Ok(rendered)
}

fn is_shadowed(
    candidate: &ReductionCandidate,
    candidates: &[ReductionCandidate],
    disabled: &BTreeSet<usize>,
) -> bool {
    let mut parent_id = candidate.parent_id;
    while let Some(id) = parent_id {
        if disabled.contains(&id) {
            return true;
        }
        parent_id = candidates.get(id).and_then(|candidate| candidate.parent_id);
    }
    false
}

fn merge_spans(spans: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    let mut merged = Vec::new();
    for (start, end) in spans {
        if let Some((_, last_end)) = merged.last_mut() {
            if start <= *last_end {
                *last_end = (*last_end).max(end);
                continue;
            }
        }
        merged.push((start, end));
    }
    merged
}

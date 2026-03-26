use anyhow::{anyhow, Result};
use std::collections::BTreeSet;

use crate::model::{CandidateKind, ReductionCandidate};
use crate::profile;

pub fn render_source(
    source: &str,
    candidates: &[ReductionCandidate],
    disabled: &BTreeSet<usize>,
) -> Result<String> {
    let _scope = profile::Scope::new("render::render_source");
    let mut spans = Vec::new();
    for id in disabled {
        let candidate = candidates
            .get(*id)
            .ok_or_else(|| anyhow!("unknown candidate id {id}"))?;
        if is_shadowed(candidate, candidates, disabled) {
            continue;
        }
        spans.push(removal_span(source, candidate));
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

fn removal_span(source: &str, candidate: &ReductionCandidate) -> (usize, usize) {
    match candidate.kind {
        CandidateKind::Node | CandidateKind::Statement | CandidateKind::CaseItem => {
            candidate.span()
        }
        CandidateKind::Port => comma_separated_removal_span(source, candidate),
        CandidateKind::DeclarationItem => declaration_item_removal_span(source, candidate),
    }
}

fn declaration_item_removal_span(source: &str, candidate: &ReductionCandidate) -> (usize, usize) {
    let item_span = comma_separated_removal_span(source, candidate);
    let bytes = source.as_bytes();

    let mut left = candidate.start;
    while left > 0 && matches!(bytes[left - 1], b' ' | b'\t') {
        left -= 1;
    }
    let has_left_comma = left > 0 && bytes[left - 1] == b',';

    let mut right = candidate.end;
    while right < bytes.len() && matches!(bytes[right], b' ' | b'\t') {
        right += 1;
    }
    let has_right_comma = right < bytes.len() && bytes[right] == b',';

    if has_left_comma || has_right_comma {
        return item_span;
    }

    let mut start = candidate.start;
    while start > 0 && bytes[start - 1] != b'\n' && bytes[start - 1] != b';' {
        start -= 1;
    }
    let mut end = candidate.end;
    while end < bytes.len() && bytes[end] != b';' {
        end += 1;
    }
    if end < bytes.len() {
        end += 1;
    }
    while end < bytes.len() && matches!(bytes[end], b' ' | b'\t' | b'\r' | b'\n') {
        end += 1;
    }
    (start, end)
}

fn comma_separated_removal_span(source: &str, candidate: &ReductionCandidate) -> (usize, usize) {
    let (mut start, mut end) = candidate.span();
    let bytes = source.as_bytes();

    let mut cursor = end;
    while cursor < bytes.len() && matches!(bytes[cursor], b' ' | b'\t' | b'\r' | b'\n') {
        cursor += 1;
    }
    if cursor < bytes.len() && bytes[cursor] == b',' {
        end = cursor + 1;
        while end < bytes.len() && matches!(bytes[end], b' ' | b'\t' | b'\r' | b'\n') {
            end += 1;
        }
        return (start, end);
    }

    cursor = start;
    while cursor > 0 && matches!(bytes[cursor - 1], b' ' | b'\t' | b'\r' | b'\n') {
        cursor -= 1;
    }
    if cursor > 0 && bytes[cursor - 1] == b',' {
        start = cursor - 1;
    }

    (start, end)
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

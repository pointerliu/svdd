use std::collections::BTreeSet;
use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use sv_parser::{parse_sv_str, NodeEvent, RefNode};

use crate::model::{CandidateKind, ReductionCandidate};

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

    cleanup_null_statements(&rendered)
}

fn removal_span(source: &str, candidate: &ReductionCandidate) -> (usize, usize) {
    match candidate.kind {
        CandidateKind::Node => candidate.span(),
        CandidateKind::Port => port_removal_span(source, candidate),
    }
}

fn port_removal_span(source: &str, candidate: &ReductionCandidate) -> (usize, usize) {
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

fn cleanup_null_statements(source: &str) -> Result<String> {
    let parse = parse_sv_str(
        source,
        &PathBuf::from("render_cleanup.sv"),
        &HashMap::new(),
        &Vec::<PathBuf>::new(),
        false,
        false,
    );

    let Ok((syntax_tree, _)) = parse else {
        return Ok(source.to_string());
    };

    let mut spans = Vec::new();
    let mut current_span: Option<(usize, usize)> = None;

    for event in syntax_tree.into_iter().event() {
        match event {
            NodeEvent::Enter(RefNode::StatementOrNullAttribute(_)) => {
                current_span = Some((usize::MAX, 0));
            }
            NodeEvent::Enter(RefNode::Locate(loc)) => {
                if let Some((start, end)) = &mut current_span {
                    if *start == usize::MAX || loc.offset < *start {
                        *start = loc.offset;
                    }
                    *end = (*end).max(loc.offset + loc.len);
                }
            }
            NodeEvent::Leave(RefNode::StatementOrNullAttribute(_)) => {
                if let Some((start, end)) = current_span.take() {
                    if start != usize::MAX && source[start..end].trim() == ";" {
                        spans.push((start, end));
                    }
                }
            }
            _ => {}
        }
    }

    if spans.is_empty() {
        return Ok(source.to_string());
    }

    let merged = merge_spans(spans);
    let mut rendered = String::with_capacity(source.len());
    let mut cursor = 0usize;
    for (start, end) in merged {
        rendered.push_str(&source[cursor..start]);
        cursor = end;
    }
    rendered.push_str(&source[cursor..]);
    Ok(rendered)
}

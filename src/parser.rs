use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use sv_parser::{parse_sv, parse_sv_str, NodeEvent, RefNode};

use crate::model::{CandidateKind, ReductionCandidate};
use crate::profile;

#[derive(Debug, Clone)]
pub struct ParsedSource {
    pub path: PathBuf,
    pub source: String,
    pub candidates: Vec<ReductionCandidate>,
    pub parse_elapsed: Duration,
}

impl ParsedSource {
    pub fn parse_file(path: impl AsRef<Path>) -> Result<Self> {
        let _scope = profile::Scope::new("parser::ParsedSource::parse_file");
        let path = path.as_ref().to_path_buf();
        let start = Instant::now();
        let (syntax_tree, _) =
            parse_sv(&path, &HashMap::new(), &Vec::<PathBuf>::new(), false, false)
                .with_context(|| format!("failed to parse {}", path.display()))?;
        let source = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let candidates = extract_candidates(&syntax_tree, &source, source.len());

        Ok(Self {
            path,
            source,
            candidates,
            parse_elapsed: start.elapsed(),
        })
    }

    pub fn parse_str(source: &str, path: impl AsRef<Path>) -> Result<Self> {
        let _scope = profile::Scope::new("parser::ParsedSource::parse_str");
        let path = path.as_ref().to_path_buf();
        let start = Instant::now();
        let (syntax_tree, _) = parse_sv_str(
            source,
            &path,
            &HashMap::new(),
            &Vec::<PathBuf>::new(),
            false,
            false,
        )
        .with_context(|| format!("failed to parse {}", path.display()))?;
        let candidates = extract_candidates(&syntax_tree, source, source.len());

        Ok(Self {
            path,
            source: source.to_owned(),
            candidates,
            parse_elapsed: start.elapsed(),
        })
    }
}

fn extract_candidates(
    syntax_tree: &sv_parser::SyntaxTree,
    source: &str,
    source_len: usize,
) -> Vec<ReductionCandidate> {
    let _scope = profile::Scope::new("parser::extract_candidates");
    let mut candidates = Vec::<ReductionCandidate>::new();
    let mut stack = Vec::<usize>::new();

    for event in syntax_tree.into_iter().event() {
        match event {
            NodeEvent::Enter(RefNode::WhiteSpace(_)) => {}
            NodeEvent::Leave(RefNode::WhiteSpace(_)) => {}
            NodeEvent::Enter(RefNode::Locate(loc)) => {
                for id in &stack {
                    candidates[*id].touch(loc.offset, loc.len);
                }
            }
            NodeEvent::Leave(RefNode::Locate(_)) => {}
            NodeEvent::Enter(node) if is_removable_node(node.clone()) => {
                let id = candidates.len();
                let parent_id = stack.last().copied();
                let mut candidate = ReductionCandidate::with_parent(
                    id,
                    candidate_kind(node.clone()),
                    usize::MAX,
                    0,
                    parent_id,
                );
                candidate.depth = stack.len();
                candidates.push(candidate);
                if let Some(parent_id) = parent_id {
                    candidates[parent_id].children.push(id);
                }
                stack.push(id);
            }
            NodeEvent::Enter(_) => {}
            NodeEvent::Leave(node) if is_removable_node(node.clone()) => {
                stack.pop();
            }
            NodeEvent::Leave(_) => {}
        }
    }

    let mut candidates = remap_candidates(candidates, source_len);
    annotate_identifiers(&mut candidates, source);
    candidates
}

fn annotate_identifiers(candidates: &mut [ReductionCandidate], source: &str) {
    for candidate in candidates {
        let span = &source[candidate.start..candidate.end];
        candidate.provided_identifiers = match candidate.kind {
            CandidateKind::Port => extract_last_identifier(span).into_iter().collect(),
            CandidateKind::DeclarationItem => extract_first_identifier(span).into_iter().collect(),
            _ => Vec::new(),
        };
    }
}

fn extract_first_identifier(span: &str) -> Option<String> {
    identifier_tokens(span).into_iter().next()
}

fn extract_last_identifier(span: &str) -> Option<String> {
    identifier_tokens(span).into_iter().last()
}

fn identifier_tokens(span: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in span.chars() {
        if ch == '_' || ch.is_ascii_alphanumeric() {
            current.push(ch);
        } else if !current.is_empty() {
            if current.chars().next().unwrap().is_ascii_alphabetic() || current.starts_with('_') {
                tokens.push(current.clone());
            }
            current.clear();
        }
    }
    if !current.is_empty()
        && (current.chars().next().unwrap().is_ascii_alphabetic() || current.starts_with('_'))
    {
        tokens.push(current);
    }
    tokens
}

fn is_removable_node(node: RefNode<'_>) -> bool {
    matches!(
        node,
        RefNode::NetDeclAssignment(_)
            | RefNode::VariableDeclAssignmentVariable(_)
            | RefNode::VariableDeclAssignmentDynamicArray(_)
            | RefNode::VariableDeclAssignmentClass(_)
            | RefNode::ContinuousAssign(_)
            | RefNode::InitialConstruct(_)
            | RefNode::AlwaysConstruct(_)
            | RefNode::Statement(_)
            | RefNode::CaseStatement(_)
            | RefNode::CaseItemNondefault(_)
            | RefNode::CaseItemDefault(_)
            | RefNode::ConditionalStatement(_)
            | RefNode::AnsiPortDeclarationNet(_)
            | RefNode::AnsiPortDeclarationVariable(_)
            | RefNode::AnsiPortDeclarationParen(_)
            | RefNode::PortDeclarationInout(_)
            | RefNode::PortDeclarationInput(_)
            | RefNode::PortDeclarationOutput(_)
            | RefNode::PortDeclarationRef(_)
            | RefNode::PortDeclarationInterface(_)
    )
}

fn candidate_kind(node: RefNode<'_>) -> CandidateKind {
    match node {
        RefNode::AnsiPortDeclarationNet(_)
        | RefNode::AnsiPortDeclarationVariable(_)
        | RefNode::AnsiPortDeclarationParen(_)
        | RefNode::PortDeclarationInout(_)
        | RefNode::PortDeclarationInput(_)
        | RefNode::PortDeclarationOutput(_)
        | RefNode::PortDeclarationRef(_)
        | RefNode::PortDeclarationInterface(_) => CandidateKind::Port,
        RefNode::Statement(_) => CandidateKind::Statement,
        RefNode::NetDeclAssignment(_)
        | RefNode::VariableDeclAssignmentVariable(_)
        | RefNode::VariableDeclAssignmentDynamicArray(_)
        | RefNode::VariableDeclAssignmentClass(_) => CandidateKind::DeclarationItem,
        RefNode::CaseItemNondefault(_) | RefNode::CaseItemDefault(_) => CandidateKind::CaseItem,
        _ => CandidateKind::Node,
    }
}

fn remap_candidates(
    candidates: Vec<ReductionCandidate>,
    source_len: usize,
) -> Vec<ReductionCandidate> {
    let kept_ids: Vec<usize> = candidates
        .iter()
        .filter(|candidate| {
            candidate.is_span_initialized()
                && !(candidate.start == 0 && candidate.end == source_len)
        })
        .map(|candidate| candidate.id)
        .collect();

    let id_map: HashMap<usize, usize> = kept_ids
        .iter()
        .enumerate()
        .map(|(new_id, old_id)| (*old_id, new_id))
        .collect();

    kept_ids
        .into_iter()
        .map(|old_id| {
            let old = &candidates[old_id];
            let mut candidate = old.clone();
            candidate.id = id_map[&old_id];
            candidate.parent_id = remap_parent_id(old.parent_id, &candidates, &id_map);
            candidate.children = old
                .children
                .iter()
                .filter_map(|child_id| id_map.get(child_id).copied())
                .collect();
            candidate
        })
        .collect()
}

fn remap_parent_id(
    mut parent_id: Option<usize>,
    candidates: &[ReductionCandidate],
    id_map: &HashMap<usize, usize>,
) -> Option<usize> {
    while let Some(id) = parent_id {
        if let Some(mapped_id) = id_map.get(&id) {
            return Some(*mapped_id);
        }
        parent_id = candidates[id].parent_id;
    }
    None
}

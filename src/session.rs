use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Result;

use crate::check::{CheckOutcome, Checker};
use crate::cli::SyntaxCheckMode;
use crate::metrics::{AttemptMetrics, PerformanceMetrics};
use crate::model::{ReductionCandidate, ReductionSummary};
use crate::parser::ParsedSource;
use crate::profile;
use crate::render::render_source;

#[derive(Debug, Clone)]
pub struct SessionInput {
    pub source: String,
    pub candidates: Vec<ReductionCandidate>,
}

impl SessionInput {
    pub fn new(source: String, candidates: Vec<ReductionCandidate>) -> Self {
        Self { source, candidates }
    }
}

pub struct ReductionSession {
    input: SessionInput,
    checker: Box<dyn Checker>,
    metrics: PerformanceMetrics,
    attempt_cache: BTreeMap<Vec<usize>, bool>,
    rendered_cache: BTreeMap<String, bool>,
    candidate_order: Vec<usize>,
    sibling_groups: Vec<Vec<usize>>,
    output_dir: Option<PathBuf>,
    input_file_name: Option<String>,
    attempt_index: usize,
    validation_path: Option<String>,
    syntax_check: SyntaxCheckMode,
}

impl ReductionSession {
    pub fn new<C>(input: SessionInput, checker: C) -> Self
    where
        C: Checker + 'static,
    {
        let candidate_order = build_candidate_order(&input.candidates);
        let sibling_groups = build_sibling_groups(&input.candidates);
        Self {
            input,
            checker: Box::new(checker),
            metrics: PerformanceMetrics::default(),
            attempt_cache: BTreeMap::new(),
            rendered_cache: BTreeMap::new(),
            candidate_order,
            sibling_groups,
            output_dir: None,
            input_file_name: None,
            attempt_index: 0,
            validation_path: None,
            syntax_check: SyntaxCheckMode::Off,
        }
    }

    pub fn with_output_dir(mut self, output_dir: PathBuf, input_path: impl AsRef<Path>) -> Self {
        self.output_dir = Some(output_dir);
        self.input_file_name = input_path
            .as_ref()
            .file_name()
            .map(|value| value.to_string_lossy().into_owned());
        self
    }

    pub fn with_parse_validation(
        mut self,
        validation_path: impl Into<String>,
        syntax_check: SyntaxCheckMode,
    ) -> Self {
        self.validation_path = Some(validation_path.into());
        self.syntax_check = syntax_check;
        self
    }

    pub fn metrics_mut(&mut self) -> &mut PerformanceMetrics {
        &mut self.metrics
    }

    pub fn candidate_ids(&self) -> impl Iterator<Item = usize> + '_ {
        self.candidate_order.iter().copied()
    }

    pub fn grouped_siblings(&self) -> Vec<Vec<usize>> {
        self.sibling_groups.clone()
    }

    pub fn depths(&self) -> Vec<usize> {
        let mut depths: Vec<usize> = self
            .input
            .candidates
            .iter()
            .map(|candidate| candidate.depth)
            .collect();
        depths.sort_unstable();
        depths.dedup();
        depths
    }

    pub fn level_candidate_ids(&self, depth: usize, disabled: &BTreeSet<usize>) -> Vec<usize> {
        self.candidate_order
            .iter()
            .copied()
            .filter(|id| {
                let candidate = &self.input.candidates[*id];
                candidate.depth == depth
                    && !disabled.contains(id)
                    && !self.has_disabled_ancestor(*id, disabled)
            })
            .collect()
    }

    pub fn attempt_disable(
        &mut self,
        disabled: &mut BTreeSet<usize>,
        ids: &[usize],
    ) -> Result<bool> {
        let _scope = profile::Scope::new("session::ReductionSession::attempt_disable");
        let start = Instant::now();
        let mut trial_disabled = disabled.clone();
        trial_disabled.extend(ids.iter().copied());
        let cache_key: Vec<usize> = trial_disabled.iter().copied().collect();

        if let Some(accepted) = self.attempt_cache.get(&cache_key).copied() {
            self.metrics.record_attempt(AttemptMetrics {
                accepted,
                duration: start.elapsed(),
            });
            if accepted {
                disabled.extend(ids.iter().copied());
            }
            return Ok(accepted);
        }

        let render_start = Instant::now();
        let rendered = render_source(&self.input.source, &self.input.candidates, &trial_disabled)?;
        self.metrics.render_elapsed += render_start.elapsed();

        if self.has_broken_identifier_dependencies(ids, &rendered) {
            self.attempt_cache.insert(cache_key, false);
            self.rendered_cache.insert(rendered, false);
            self.metrics.record_attempt(AttemptMetrics {
                accepted: false,
                duration: start.elapsed(),
            });
            return Ok(false);
        }

        if let Some(accepted) = self.rendered_cache.get(&rendered).copied() {
            self.attempt_cache.insert(cache_key, accepted);
            self.metrics.record_attempt(AttemptMetrics {
                accepted,
                duration: start.elapsed(),
            });
            if accepted {
                disabled.extend(ids.iter().copied());
            }
            return Ok(accepted);
        }

        let rendered_path = self.persist_attempt(&rendered)?;

        if self.syntax_check == SyntaxCheckMode::Always {
            let validation_path = self.validation_path.as_deref().unwrap_or("trial.sv");
            let parse_start = Instant::now();
            let parse_result = ParsedSource::parse_str(&rendered, validation_path);
            self.metrics.parse_elapsed += parse_start.elapsed();
            if parse_result.is_err() {
                self.attempt_cache.insert(cache_key, false);
                self.rendered_cache.insert(rendered, false);
                self.metrics.record_attempt(AttemptMetrics {
                    accepted: false,
                    duration: start.elapsed(),
                });
                return Ok(false);
            }
        }

        let check_start = Instant::now();
        let outcome = if let Some(path) = rendered_path.as_deref() {
            self.checker.check_path(path, &rendered, &trial_disabled)?
        } else {
            self.checker.check(&rendered, &trial_disabled)?
        };
        self.metrics.check_elapsed += check_start.elapsed();

        let accepted = matches!(outcome, CheckOutcome::Kept);
        self.attempt_cache.insert(cache_key, accepted);
        self.rendered_cache.insert(rendered, accepted);
        self.metrics.record_attempt(AttemptMetrics {
            accepted,
            duration: start.elapsed(),
        });

        if accepted {
            disabled.extend(ids.iter().copied());
        }

        Ok(accepted)
    }

    pub fn finalize(
        mut self,
        disabled_candidates: BTreeSet<usize>,
        total_elapsed: std::time::Duration,
    ) -> ReductionSummary {
        self.metrics.total_elapsed = total_elapsed;
        ReductionSummary {
            disabled_candidates,
            metrics: self.metrics,
        }
    }

    fn persist_attempt(&mut self, rendered: &str) -> Result<Option<PathBuf>> {
        let _scope = profile::Scope::new("session::ReductionSession::persist_attempt");
        let Some(output_dir) = &self.output_dir else {
            return Ok(None);
        };

        self.attempt_index += 1;
        let attempt_dir = output_dir
            .join("attempts")
            .join(format!("attempt-{number:06}", number = self.attempt_index));
        std::fs::create_dir_all(&attempt_dir)?;
        let path = attempt_dir.join(self.input_file_name.as_deref().unwrap_or("reduced.sv"));
        std::fs::write(&path, rendered)?;
        Ok(Some(path))
    }

    pub fn final_output_path(&self) -> Option<PathBuf> {
        self.output_dir.as_ref().map(|output_dir| {
            output_dir.join(
                self.input_file_name
                    .clone()
                    .unwrap_or_else(|| "reduced.sv".to_string()),
            )
        })
    }

    fn has_disabled_ancestor(&self, mut id: usize, disabled: &BTreeSet<usize>) -> bool {
        let mut parent_id = self.input.candidates[id].parent_id;
        while let Some(parent) = parent_id {
            if disabled.contains(&parent) {
                return true;
            }
            id = parent;
            parent_id = self.input.candidates[id].parent_id;
        }
        false
    }

    fn has_broken_identifier_dependencies(&self, ids: &[usize], rendered: &str) -> bool {
        ids.iter().any(|id| {
            self.input.candidates[*id]
                .provided_identifiers
                .iter()
                .any(|identifier| contains_identifier(rendered, identifier))
        })
    }
}

fn contains_identifier(source: &str, needle: &str) -> bool {
    let bytes = source.as_bytes();
    let needle = needle.as_bytes();
    if needle.is_empty() || needle.len() > bytes.len() {
        return false;
    }
    for start in 0..=bytes.len() - needle.len() {
        if &bytes[start..start + needle.len()] != needle {
            continue;
        }
        let left_ok = start == 0 || !is_identifier_byte(bytes[start - 1]);
        let right = start + needle.len();
        let right_ok = right == bytes.len() || !is_identifier_byte(bytes[right]);
        if left_ok && right_ok {
            return true;
        }
    }
    false
}

fn is_identifier_byte(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphanumeric()
}

fn build_candidate_order(candidates: &[ReductionCandidate]) -> Vec<usize> {
    let mut ids: Vec<usize> = candidates.iter().map(|candidate| candidate.id).collect();
    ids.sort_by_key(|id| {
        let candidate = &candidates[*id];
        (usize::MAX - candidate.depth, candidate.start)
    });
    ids
}

fn build_sibling_groups(candidates: &[ReductionCandidate]) -> Vec<Vec<usize>> {
    let mut groups = BTreeMap::<Option<usize>, Vec<usize>>::new();
    for candidate in candidates {
        groups
            .entry(candidate.parent_id)
            .or_default()
            .push(candidate.id);
    }
    let mut groups: Vec<Vec<usize>> = groups
        .into_values()
        .filter(|group| group.len() > 1)
        .collect();
    groups.sort_by_key(|group| {
        let depth = group
            .iter()
            .map(|id| candidates[*id].depth)
            .max()
            .unwrap_or(0);
        usize::MAX - depth
    });
    groups
}

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Result;

use crate::check::{CheckOutcome, Checker};
use crate::metrics::{AttemptMetrics, PerformanceMetrics};
use crate::model::{ReductionCandidate, ReductionSummary};
use crate::parser::ParsedSource;
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
    output_dir: Option<PathBuf>,
    input_file_name: Option<String>,
    attempt_index: usize,
    validation_path: Option<String>,
}

impl ReductionSession {
    pub fn new<C>(input: SessionInput, checker: C) -> Self
    where
        C: Checker + 'static,
    {
        Self {
            input,
            checker: Box::new(checker),
            metrics: PerformanceMetrics::default(),
            output_dir: None,
            input_file_name: None,
            attempt_index: 0,
            validation_path: None,
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

    pub fn with_parse_validation(mut self, validation_path: impl Into<String>) -> Self {
        self.validation_path = Some(validation_path.into());
        self
    }

    pub fn metrics_mut(&mut self) -> &mut PerformanceMetrics {
        &mut self.metrics
    }

    pub fn candidate_ids(&self) -> impl Iterator<Item = usize> + '_ {
        self.input.candidates.iter().map(|candidate| candidate.id)
    }

    pub fn grouped_siblings(&self) -> Vec<Vec<usize>> {
        let mut groups = BTreeMap::<Option<usize>, Vec<usize>>::new();
        for candidate in &self.input.candidates {
            groups
                .entry(candidate.parent_id)
                .or_default()
                .push(candidate.id);
        }
        groups
            .into_values()
            .filter(|group| group.len() > 1)
            .collect()
    }

    pub fn attempt_disable(
        &mut self,
        disabled: &mut BTreeSet<usize>,
        ids: &[usize],
    ) -> Result<bool> {
        let start = Instant::now();
        let mut trial_disabled = disabled.clone();
        trial_disabled.extend(ids.iter().copied());

        let render_start = Instant::now();
        let rendered = render_source(&self.input.source, &self.input.candidates, &trial_disabled)?;
        self.metrics.render_elapsed += render_start.elapsed();

        let rendered_path = self.persist_attempt(&rendered)?;

        if let Some(validation_path) = &self.validation_path {
            let parse_start = Instant::now();
            let parse_result = ParsedSource::parse_str(&rendered, validation_path);
            self.metrics.parse_elapsed += parse_start.elapsed();
            if parse_result.is_err() {
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
        let Some(output_dir) = &self.output_dir else {
            return Ok(None);
        };

        self.attempt_index += 1;
        let attempts_dir = output_dir.join("attempts");
        std::fs::create_dir_all(&attempts_dir)?;
        let path = attempts_dir.join(format!(
            "attempt-{number:06}.sv",
            number = self.attempt_index
        ));
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
}

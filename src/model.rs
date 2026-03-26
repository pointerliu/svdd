use std::collections::BTreeSet;

use crate::metrics::PerformanceMetrics;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CandidateKind {
    Node,
    Statement,
    Port,
    DeclarationItem,
    CaseItem,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReductionCandidate {
    pub id: usize,
    pub kind: CandidateKind,
    pub start: usize,
    pub end: usize,
    pub line_count: usize,
    pub depth: usize,
    pub parent_id: Option<usize>,
    pub children: Vec<usize>,
    pub provided_identifiers: Vec<String>,
}

impl ReductionCandidate {
    pub fn new(id: usize, kind: CandidateKind, start: usize, end: usize) -> Self {
        Self {
            id,
            kind,
            start,
            end,
            line_count: 1,
            depth: 0,
            parent_id: None,
            children: Vec::new(),
            provided_identifiers: Vec::new(),
        }
    }

    pub fn with_parent(
        id: usize,
        kind: CandidateKind,
        start: usize,
        end: usize,
        parent_id: Option<usize>,
    ) -> Self {
        Self {
            id,
            kind,
            start,
            end,
            line_count: 1,
            depth: 0,
            parent_id,
            children: Vec::new(),
            provided_identifiers: Vec::new(),
        }
    }

    pub fn span(&self) -> (usize, usize) {
        (self.start, self.end)
    }

    pub fn touch(&mut self, offset: usize, len: usize) {
        if self.start == usize::MAX || offset < self.start {
            self.start = offset;
        }
        let end = offset + len;
        if end > self.end {
            self.end = end;
        }
    }

    pub fn is_span_initialized(&self) -> bool {
        self.start != usize::MAX && self.end >= self.start
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReductionSummary {
    pub disabled_candidates: BTreeSet<usize>,
    pub metrics: PerformanceMetrics,
}

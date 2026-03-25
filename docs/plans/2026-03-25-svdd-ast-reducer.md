# svdd AST Reducer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust CLI that minimizes a single SystemVerilog entry file by removing AST-backed elements while preserving a user-defined property checked by an external script.

**Architecture:** Parse the source once with `sv-parser`, extract reducible AST candidates with stable IDs and hierarchy metadata, and let pluggable reduction algorithms decide which candidates to disable. Rendering stays AST-driven: the tool emits reduced source from accepted candidate spans and never performs reduction decisions on raw text. The execution engine runs the user script after each attempt and records algorithm performance metrics so time cost is visible and comparable across strategies.

**Tech Stack:** Rust, `sv-parser`, `clap`, `anyhow`, `tempfile`

---

### Task 1: Bootstrap crate and CLI surface

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/main.rs`
- Test: `tests/cli_config.rs`

**Step 1: Write the failing test**

Write a test that parses CLI options into a config with:
- input path
- check script path
- algorithm name
- output path

**Step 2: Run test to verify it fails**

Run: `cargo test cli_config -- --nocapture`
Expected: compile failure because crate and CLI config do not exist yet.

**Step 3: Write minimal implementation**

Implement `Cli`/`Config` parsing with defaults:
- default algorithm: `naive`
- optional output path
- required input file and script path

**Step 4: Run test to verify it passes**

Run: `cargo test cli_config -- --nocapture`
Expected: PASS.

### Task 2: Add reduction domain model and performance metrics

**Files:**
- Create: `src/model.rs`
- Create: `src/metrics.rs`
- Test: `tests/metrics.rs`

**Step 1: Write the failing test**

Write tests covering:
- attempt durations are accumulated
- accepted/rejected attempt counts are tracked
- final summary includes total wall-clock time and per-phase time (`parse`, `render`, `check`, `algorithm`)

**Step 2: Run test to verify it fails**

Run: `cargo test metrics -- --nocapture`
Expected: compile failure because metrics types do not exist yet.

**Step 3: Write minimal implementation**

Implement:
- `ReductionCandidate`
- `CandidateKind`
- `ReductionSummary`
- `AttemptMetrics`
- `PerformanceMetrics`

Include algorithm time cost fields:
- `total_elapsed`
- `parse_elapsed`
- `render_elapsed`
- `check_elapsed`
- `algorithm_elapsed`
- `attempt_count`
- `accepted_attempts`
- `rejected_attempts`

**Step 4: Run test to verify it passes**

Run: `cargo test metrics -- --nocapture`
Expected: PASS.

### Task 3: Parse SystemVerilog and extract AST-backed candidates

**Files:**
- Create: `src/parser.rs`
- Test: `tests/parser_candidates.rs`

**Step 1: Write the failing test**

Write a test that parses a tiny module and asserts:
- at least one reducible candidate exists
- candidates have stable IDs in traversal order
- parent/child relationships are populated for nested nodes

**Step 2: Run test to verify it fails**

Run: `cargo test parser_candidates -- --nocapture`
Expected: compile failure because parser wrapper and extractor do not exist yet.

**Step 3: Write minimal implementation**

Implement parser wrapper around `sv_parser::parse_sv_str` and candidate extraction based on AST node locations. Ignore whitespace-only nodes and root-only spans. Track:
- source offset span
- candidate depth
- optional parent ID
- child IDs

**Step 4: Run test to verify it passes**

Run: `cargo test parser_candidates -- --nocapture`
Expected: PASS.

### Task 4: Render reduced source from disabled AST candidates

**Files:**
- Create: `src/render.rs`
- Test: `tests/render.rs`

**Step 1: Write the failing test**

Write tests covering:
- disabling a candidate removes its span from rendered output
- disabling a parent suppresses descendants without double-counting
- untouched text remains byte-for-byte identical

**Step 2: Run test to verify it fails**

Run: `cargo test render -- --nocapture`
Expected: compile failure because renderer does not exist yet.

**Step 3: Write minimal implementation**

Implement renderer that merges disabled candidate spans, removes overlaps, and rebuilds source from original text slices derived from AST locations.

**Step 4: Run test to verify it passes**

Run: `cargo test render -- --nocapture`
Expected: PASS.

### Task 5: Add property check execution engine

**Files:**
- Create: `src/check.rs`
- Test: `tests/check.rs`

**Step 1: Write the failing test**

Write tests covering:
- script exit code `1` means property kept
- script exit code `0` means property lost
- other exit codes return an execution error

**Step 2: Run test to verify it fails**

Run: `cargo test check -- --nocapture`
Expected: compile failure because checker does not exist yet.

**Step 3: Write minimal implementation**

Implement a checker that writes the rendered candidate output to a temp file and invokes the user script with that file path in the environment or argv.

**Step 4: Run test to verify it passes**

Run: `cargo test check -- --nocapture`
Expected: PASS.

### Task 6: Implement pluggable reduction algorithm trait and naive reducer

**Files:**
- Create: `src/algorithms/mod.rs`
- Create: `src/algorithms/naive.rs`
- Create: `src/session.rs`
- Test: `tests/naive.rs`

**Step 1: Write the failing test**

Write a test with a fake checker proving naive reduction:
- tries candidates in stable order
- accepts removals that keep the property
- records per-attempt metrics and final totals

**Step 2: Run test to verify it fails**

Run: `cargo test naive -- --nocapture`
Expected: compile failure because algorithm/session abstractions do not exist yet.

**Step 3: Write minimal implementation**

Implement:
- `ReductionAlgorithm` trait
- `NaiveReducer`
- `ReductionSession`
- attempt loop that renders candidate state, checks property, and updates metrics

**Step 4: Run test to verify it passes**

Run: `cargo test naive -- --nocapture`
Expected: PASS.

### Task 7: Implement hierarchical delta debugging reducer

**Files:**
- Create: `src/algorithms/hdd.rs`
- Test: `tests/hdd.rs`

**Step 1: Write the failing test**

Write a test with a fake candidate tree proving HDD:
- groups siblings by level
- attempts chunk removals before leaf-by-leaf removals
- keeps performance accounting separate from naive

**Step 2: Run test to verify it fails**

Run: `cargo test hdd -- --nocapture`
Expected: compile failure because HDD reducer does not exist yet.

**Step 3: Write minimal implementation**

Implement HDD using candidate hierarchy levels and chunk partitioning over sibling groups.

**Step 4: Run test to verify it passes**

Run: `cargo test hdd -- --nocapture`
Expected: PASS.

### Task 8: Wire CLI end-to-end and report performance

**Files:**
- Modify: `src/main.rs`
- Create: `tests/end_to_end.rs`

**Step 1: Write the failing test**

Write an end-to-end test that runs the tool on a small fixture and asserts stdout reports:
- selected algorithm
- number of attempts
- accepted/rejected counts
- total reduction time
- breakdown for parse/render/check/algorithm time cost

**Step 2: Run test to verify it fails**

Run: `cargo test end_to_end -- --nocapture`
Expected: FAIL because CLI execution/reporting is incomplete.

**Step 3: Write minimal implementation**

Wire algorithm selection, output writing, and human-readable metric reporting.

**Step 4: Run test to verify it passes**

Run: `cargo test end_to_end -- --nocapture`
Expected: PASS.

### Task 9: Final verification

**Files:**
- Verify: `Cargo.toml`
- Verify: `src/**/*.rs`
- Verify: `tests/*.rs`

**Step 1: Run focused tests**

Run: `cargo test`
Expected: all tests pass.

**Step 2: Run formatting and lint-like verification**

Run: `cargo fmt -- --check && cargo check`
Expected: success.

**Step 3: Review architecture against requirements**

Confirm:
- reductions are decided on AST candidates, not raw text search
- algorithms are pluggable
- time cost is reported per algorithm run and per phase
- single-entry-file v1 is explicit

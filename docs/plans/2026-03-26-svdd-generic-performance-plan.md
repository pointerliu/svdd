# SVDD Generic Performance Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Reduce end-to-end runtime for AST-based SystemVerilog reduction without adding demo-specific heuristics.

**Architecture:** Move prioritization into shared candidate metadata so all reducers can benefit from a better traversal order. Reject syntactically invalid intermediate sources before invoking expensive external checkers, and skip attempts that are already shadowed by accepted ancestor removals.

**Tech Stack:** Rust, `sv-parser`, clap, external script-based checkers.

---

### Task 1: Add candidate size metadata

**Files:**
- Modify: `src/model.rs`
- Modify: `src/parser.rs`
- Test: `tests/naive.rs`

**Step 1:** Add a failing test that expects large candidates to be tried before smaller descendants.

**Step 2:** Run `cargo test --test naive` and confirm it fails.

**Step 3:** Add `line_count` to `ReductionCandidate` and compute it from parsed source spans.

**Step 4:** Re-run `cargo test --test naive` and confirm the new ordering test passes.

### Task 2: Reuse size-aware ordering across reducers

**Files:**
- Modify: `src/session.rs`
- Modify: `src/algorithms/naive.rs`
- Modify: `src/algorithms/hdd.rs`
- Test: `tests/naive.rs`

**Step 1:** Add a failing test that proves candidates shadowed by an already-disabled ancestor are skipped.

**Step 2:** Run `cargo test --test naive` and confirm it fails.

**Step 3:** Update shared candidate ordering in `ReductionSession` to sort by line count first, then depth, and expose a helper that filters out shadowed candidates.

**Step 4:** Use that helper in `naive` and `hdd` so accepted parent removals do not trigger redundant child attempts.

**Step 5:** Re-run `cargo test --test naive` and `cargo test`.

### Task 3: Reduce external checker cost generically

**Files:**
- Modify: `src/cli.rs`
- Test: `tests/cli_config.rs`
- Verify: `tests/session_validation.rs`

**Step 1:** Add a failing test that expects syntax validation to be enabled by default.

**Step 2:** Run `cargo test --test cli_config` and confirm it fails.

**Step 3:** Change the CLI default to `--syntax-check always` so invalid intermediate reductions are rejected before spawning external tools.

**Step 4:** Re-run `cargo test` and confirm all tests pass.

### Task 4: Verify with the real demo

**Files:**
- Verify: `duts/bugpoint_demo/compute_unit.sv`
- Verify: `duts/bugpoint_demo/sv-bugpoint-check.sh`

**Step 1:** Build `svdd` in release mode with `cargo build --release`.

**Step 2:** Time `sv-bugpoint` on the demo using `/usr/bin/time`.

**Step 3:** Time `svdd --algorithm naive` on the same demo using the same check script and matching keep/reject exit codes.

**Step 4:** Compare elapsed time and attempt counts, then keep only generic improvements that preserve correctness.

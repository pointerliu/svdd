use std::collections::BTreeSet;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use svdd::algorithms::{
    ddmin::DdminReducer, hdd::HddReducer, hddmin::HddminReducer, naive::NaiveReducer,
    ReductionAlgorithm,
};
use svdd::check::CheckOutcome;
use svdd::parser::ParsedSource;
use svdd::session::{ReductionSession, SessionInput};

fn bench_demo_algorithms(c: &mut Criterion) {
    let parsed = ParsedSource::parse_file("examples/bugpoint_demo/compute_unit.sv").unwrap();

    c.bench_function("demo naive reducer", |b| {
        b.iter(|| {
            let session = ReductionSession::new(
                SessionInput::new(parsed.source.clone(), parsed.candidates.clone()),
                demo_oracle,
            );
            let summary = NaiveReducer.run(session).unwrap();
            black_box(summary);
        });
    });

    c.bench_function("demo hdd reducer", |b| {
        b.iter(|| {
            let session = ReductionSession::new(
                SessionInput::new(parsed.source.clone(), parsed.candidates.clone()),
                demo_oracle,
            );
            let summary = HddReducer.run(session).unwrap();
            black_box(summary);
        });
    });

    c.bench_function("demo ddmin reducer", |b| {
        b.iter(|| {
            let session = ReductionSession::new(
                SessionInput::new(parsed.source.clone(), parsed.candidates.clone()),
                demo_oracle,
            );
            let summary = DdminReducer.run(session).unwrap();
            black_box(summary);
        });
    });

    c.bench_function("demo hddmin reducer", |b| {
        b.iter(|| {
            let session = ReductionSession::new(
                SessionInput::new(parsed.source.clone(), parsed.candidates.clone()),
                demo_oracle,
            );
            let summary = HddminReducer.run(session).unwrap();
            black_box(summary);
        });
    });
}

fn demo_oracle(rendered: &str, _disabled: &BTreeSet<usize>) -> CheckOutcome {
    let required = [
        "module compute_unit",
        "input wire clk",
        "input wire rst",
        "input wire [WIDTH-1:0] a",
        "input wire [WIDTH-1:0] b",
        "input wire [1:0] sel",
        "output reg [WIDTH-1:0] y",
        "always_ff @(posedge clk)",
        "case (sel)",
        "2'b11",
        "y <= a - b;",
    ];

    if required.iter().all(|needle| rendered.contains(needle)) {
        CheckOutcome::Kept
    } else {
        CheckOutcome::Lost
    }
}

criterion_group!(benches, bench_demo_algorithms);
criterion_main!(benches);

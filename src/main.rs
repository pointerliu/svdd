use anyhow::Result;
use clap::Parser;
use std::time::Instant;

use svdd::algorithms::{
    ddmin::DdminReducer, hdd::HddReducer, hddmin::HddminReducer, naive::NaiveReducer,
    ReductionAlgorithm,
};
use svdd::check::ScriptChecker;
use svdd::cli::{AlgorithmKind, Cli};
use svdd::parser::ParsedSource;
use svdd::render::render_source;
use svdd::session::{ReductionSession, SessionInput};

fn main() -> Result<()> {
    let overall_start = Instant::now();
    let cli = Cli::parse();
    let parsed = ParsedSource::parse_file(&cli.input)?;

    let mut session = ReductionSession::new(
        SessionInput::new(parsed.source.clone(), parsed.candidates.clone()),
        ScriptChecker::with_exit_codes(
            cli.check_script.clone(),
            cli.keep_exit_code,
            cli.reject_exit_code,
        ),
    )
    .with_output_dir(cli.output_dir.clone(), &parsed.path)
    .with_parse_validation(parsed.path.display().to_string());
    session.metrics_mut().parse_elapsed = parsed.parse_elapsed;

    let mut summary = match cli.algorithm {
        AlgorithmKind::Naive => NaiveReducer.run(session)?,
        AlgorithmKind::Hdd => HddReducer.run(session)?,
        AlgorithmKind::Ddmin => DdminReducer.run(session)?,
        AlgorithmKind::Hddmin => HddminReducer.run(session)?,
    };

    let reduced = render_source(
        &parsed.source,
        &parsed.candidates,
        &summary.disabled_candidates,
    )?;
    std::fs::create_dir_all(&cli.output_dir)?;
    let output_path = cli.output_dir.join(
        parsed
            .path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| "reduced.sv".to_string()),
    );
    std::fs::write(&output_path, reduced)?;
    summary.metrics.total_elapsed = overall_start.elapsed();

    println!(
        "algorithm: {}",
        match cli.algorithm {
            AlgorithmKind::Naive => "naive",
            AlgorithmKind::Hdd => "hdd",
            AlgorithmKind::Ddmin => "ddmin",
            AlgorithmKind::Hddmin => "hddmin",
        }
    );
    println!("attempts: {}", summary.metrics.attempt_count);
    println!("accepted: {}", summary.metrics.accepted_attempts);
    println!("rejected: {}", summary.metrics.rejected_attempts);
    println!("total: {:?}", summary.metrics.total_elapsed);
    println!("parse: {:?}", summary.metrics.parse_elapsed);
    println!("render: {:?}", summary.metrics.render_elapsed);
    println!("check: {:?}", summary.metrics.check_elapsed);
    println!("algorithm cost: {:?}", summary.metrics.algorithm_elapsed);
    println!("output: {}", output_path.display());

    Ok(())
}

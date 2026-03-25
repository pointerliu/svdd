use clap::Parser;

use svdd::cli::{AlgorithmKind, Cli};

#[test]
fn parses_cli_with_required_output_dir() {
    let cli = Cli::parse_from([
        "svdd",
        "input.sv",
        "--check-script",
        "./check.sh",
        "--output-dir",
        "./out",
    ]);

    assert_eq!(cli.input.as_os_str(), "input.sv");
    assert_eq!(cli.check_script.as_os_str(), "./check.sh");
    assert_eq!(cli.algorithm, AlgorithmKind::Naive);
    assert_eq!(cli.output_dir.as_os_str(), "./out");
}

#[test]
fn parses_cli_with_hdd_algorithm() {
    let cli = Cli::parse_from([
        "svdd",
        "input.sv",
        "--check-script",
        "./check.sh",
        "--algorithm",
        "hdd",
        "--output-dir",
        "out",
    ]);

    assert_eq!(cli.algorithm, AlgorithmKind::Hdd);
    assert_eq!(cli.output_dir.as_os_str(), "out");
}

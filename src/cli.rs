use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum AlgorithmKind {
    Naive,
    Hdd,
    Ddmin,
    Hddmin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SyntaxCheckMode {
    Off,
    Always,
}

#[derive(Debug, Clone, Parser)]
#[command(name = "svdd")]
#[command(about = "Reduce SystemVerilog with AST-backed candidate removal")]
pub struct Cli {
    pub input: PathBuf,

    #[arg(long)]
    pub check_script: PathBuf,

    #[arg(long, value_enum, default_value_t = AlgorithmKind::Naive)]
    pub algorithm: AlgorithmKind,

    #[arg(long)]
    pub output_dir: PathBuf,

    #[arg(long, value_enum, default_value_t = SyntaxCheckMode::Always)]
    pub syntax_check: SyntaxCheckMode,

    #[arg(long, default_value_t = 1)]
    pub keep_exit_code: i32,

    #[arg(long, default_value_t = 0)]
    pub reject_exit_code: i32,
}

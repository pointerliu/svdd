use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use tempfile::NamedTempFile;

use crate::profile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckOutcome {
    Kept,
    Lost,
}

pub trait Checker {
    fn check(&mut self, rendered: &str, disabled: &BTreeSet<usize>) -> Result<CheckOutcome>;

    fn check_path(
        &mut self,
        rendered_path: &Path,
        rendered: &str,
        disabled: &BTreeSet<usize>,
    ) -> Result<CheckOutcome> {
        let _ = rendered_path;
        self.check(rendered, disabled)
    }
}

impl<F> Checker for F
where
    F: FnMut(&str, &BTreeSet<usize>) -> CheckOutcome,
{
    fn check(&mut self, rendered: &str, disabled: &BTreeSet<usize>) -> Result<CheckOutcome> {
        Ok(self(rendered, disabled))
    }
}

#[derive(Debug, Clone)]
pub struct ScriptChecker {
    script: PathBuf,
    keep_exit_code: i32,
    reject_exit_code: i32,
}

impl ScriptChecker {
    pub fn new(script: PathBuf) -> Self {
        Self::with_exit_codes(script, 1, 0)
    }

    pub fn with_exit_codes(script: PathBuf, keep_exit_code: i32, reject_exit_code: i32) -> Self {
        Self {
            script,
            keep_exit_code,
            reject_exit_code,
        }
    }

    pub fn run(&self, rendered: &str) -> Result<CheckOutcome> {
        let _scope = profile::Scope::new("check::ScriptChecker::run");
        let mut temp =
            NamedTempFile::new().context("failed to create temp file for check script")?;
        std::io::Write::write_all(&mut temp, rendered.as_bytes())
            .context("failed to write reduced source to temp file")?;

        self.run_file(temp.path())
    }

    pub fn run_file(&self, rendered_path: &Path) -> Result<CheckOutcome> {
        let _scope = profile::Scope::new("check::ScriptChecker::run_file");
        let status = Command::new(&self.script)
            .arg(rendered_path)
            .status()
            .with_context(|| format!("failed to run check script {}", self.script.display()))?;

        match status.code() {
            Some(code) if code == self.keep_exit_code => Ok(CheckOutcome::Kept),
            Some(code) if code == self.reject_exit_code => Ok(CheckOutcome::Lost),
            Some(code) => Err(anyhow!(
                "check script exited with unexpected code {code} (keep={}, reject={})",
                self.keep_exit_code,
                self.reject_exit_code
            )),
            None => Err(anyhow!("check script terminated without an exit code")),
        }
    }
}

impl Checker for ScriptChecker {
    fn check(&mut self, rendered: &str, _disabled: &BTreeSet<usize>) -> Result<CheckOutcome> {
        self.run(rendered)
    }

    fn check_path(
        &mut self,
        rendered_path: &Path,
        _rendered: &str,
        _disabled: &BTreeSet<usize>,
    ) -> Result<CheckOutcome> {
        self.run_file(rendered_path)
    }
}

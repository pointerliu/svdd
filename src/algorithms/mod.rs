pub mod hdd;
pub mod naive;

use anyhow::Result;

use crate::model::ReductionSummary;
use crate::session::ReductionSession;

pub trait ReductionAlgorithm {
    fn name(&self) -> &'static str;
    fn run(&self, session: ReductionSession) -> Result<ReductionSummary>;
}

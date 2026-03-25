pub mod ddmin;
pub mod hdd;
pub mod hddmin;
pub mod naive;

use anyhow::Result;

use crate::model::ReductionSummary;
use crate::session::ReductionSession;

pub trait ReductionAlgorithm {
    fn name(&self) -> &'static str;
    fn run(&self, session: ReductionSession) -> Result<ReductionSummary>;
}

use super::super::contexts::SplitContext;
use serde::{Deserialize, Serialize};
use zkm_prover::InnerSC;
use zkm_recursion_circuit::machine::ZKMDeferredWitnessValues;

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SplitTask {
    pub ctx: SplitContext,
    pub state: u32,

    pub total_steps: u64,
    pub total_segments: u32,

    pub output: Vec<ZKMDeferredWitnessValues<InnerSC>>,
}

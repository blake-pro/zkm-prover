use super::super::contexts::ProveContext;
use crate::single_node_multi_prover::tasks::TASK_STATE_INITIAL;
use serde_derive::{Deserialize, Serialize};
use zkm_core_executor::ExecutionRecord;
use zkm_prover::CoreSC;
use zkm_stark::ShardProof;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProveTask {
    pub task_id: String,
    pub state: u32,
    pub ctx: ProveContext,
    pub output: Option<ShardProof<CoreSC>>,
}

impl ProveTask {
    pub fn new(
        task_id: &str,
        program_id: &str,
        elf_path: &str,
        record: ExecutionRecord,
        seg_size: u32,
    ) -> Self {
        let ctx = ProveContext {
            program_id: program_id.into(),
            elf_path: elf_path.into(),
            record,
            seg_size,
        };

        ProveTask {
            task_id: task_id.into(),
            state: TASK_STATE_INITIAL,
            ctx,
            output: None,
        }
    }
}

use crate::single_node_multi_prover::contexts::AggContext;
use crate::single_node_multi_prover::tasks::{ProveTask, TASK_STATE_UNPROCESSED};
use crate::stage::tasks::{ProveTask, Trace, TASK_STATE_SUCCESS, TASK_STATE_UNPROCESSED};
use serde::{Deserialize, Serialize};
use zkm_prover::{CoreSC, InnerSC, ZKMCircuitWitness};
use zkm_recursion_circuit::machine::{
    ZKMCompressWitnessValues, ZKMDeferredWitnessValues, ZKMRecursionWitnessValues,
};
use zkm_sdk::ZKMProof;
use zkm_stark::{StarkGenericConfig, StarkVerifyingKey, DIGEST_SIZE};

#[derive(Serialize, Deserialize)]
pub struct AggTask {
    pub task_id: String,
    pub state: u32,
    pub ctx: AggContext,
    // childs task ids
    pub childs: Vec<Option<String>>,

    pub output: Option<ZKMProof::Compressed>,
}

impl AggTask {
    pub fn clear_child_task(&mut self, task_id: &str) -> bool {
        if self.state == TASK_STATE_UNPROCESSED {
            for child in &mut self.childs {
                if let Some(t) = child {
                    if *t == task_id {
                        *child = None;
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn init_from_prove_tasks(
        vk: StarkVerifyingKey<CoreSC>,
        prove_tasks: &[ProveTask],
        is_complete: bool,
        is_first_shard: bool,
    ) -> AggTask {
        let witness = ZKMRecursionWitnessValues {
            vk,
            // will fill when prove_tasks are ready
            shard_proofs: vec![],
            is_complete,
            is_first_shard,
            // will override in prover
            vk_root: [<CoreSC as StarkGenericConfig>::Val::ZERO; DIGEST_SIZE],
        };

        AggTask {
            task_id: uuid::Uuid::new_v4().to_string(),
            state: TASK_STATE_UNPROCESSED,
            ctx: AggContext(ZKMCircuitWitness::Core(witness)),
            childs: prove_tasks
                .iter()
                .map(|t| Some(t.task_id.to_owned()))
                .collect(),
            output: None,
        }
    }

    pub fn init_from_deferred_tasks(witness: ZKMDeferredWitnessValues<InnerSC>) -> AggTask {
        AggTask {
            task_id: uuid::Uuid::new_v4().to_string(),
            state: TASK_STATE_UNPROCESSED,
            ctx: AggContext(ZKMCircuitWitness::Deferred(witness)),
            childs: vec![],
            output: None,
        }
    }

    pub fn init_from_agg_tasks(agg_tasks: &[AggTask], is_complete: bool) -> AggTask {
        let witness = ZKMCompressWitnessValues {
            // will fill when agg_tasks are ready
            vks_and_proofs: vec![],
            is_complete,
        };

        AggTask {
            task_id: uuid::Uuid::new_v4().to_string(),
            state: TASK_STATE_UNPROCESSED,
            ctx: AggContext(ZKMCircuitWitness::Compress(witness)),
            childs: agg_tasks
                .iter()
                .map(|t| Some(t.task_id.to_owned()))
                .collect(),
            output: None,
        }
    }
}

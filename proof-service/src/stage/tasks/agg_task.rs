use serde::{Deserialize, Serialize};

use crate::proto::includes::v1::AggregateInput;
use crate::stage::tasks::{ProveTask, Trace, TASK_STATE_UNPROCESSED};

pub fn from_prove_task(prove_task: &ProveTask) -> AggregateInput {
    AggregateInput {
        // we put the receipt of prove_task, instead of the file path
        receipt_input: vec![],
        computed_request_id: prove_task.task_id.clone(),
        is_agg: false,
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AggTask {
    pub task_id: String,
    pub state: u32,
    pub proof_id: String,

    // vk for zkm2 core proof
    pub vk: Vec<u8>,
    #[serde(skip_serializing, skip_deserializing)]
    pub inputs: Vec<AggregateInput>,
    pub is_final: bool,
    pub is_first_shard: bool,
    pub is_leaf_layer: bool,
    pub from_prove: bool,
    pub agg_index: i32,
    pub is_deferred: bool,

    pub trace: Trace,

    #[serde(skip_serializing, skip_deserializing)]
    pub output: Vec<u8>, // output_receipt: Vec<u8>,

    // depend
    // TODO: default value may be dangerous
    pub childs: Vec<Option<String>>,
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

    pub fn to_agg_input(&self) -> AggregateInput {
        // For the receipt_input, nothing we can do here when we create the calculation graph, so give it a default value
        AggregateInput {
            receipt_input: vec![],
            computed_request_id: self.task_id.clone(),
            is_agg: !self.from_prove,
        }
    }

    pub fn init_from_prove_tasks(
        vk: &[u8],
        prove_tasks: &[ProveTask],
        agg_index: i32,
        is_final: bool,
        is_first_shard: bool,
        is_deferred: bool,
    ) -> AggTask {
        let mut agg_task = AggTask {
            task_id: uuid::Uuid::new_v4().to_string(),
            state: TASK_STATE_UNPROCESSED,
            proof_id: prove_tasks[0].program.proof_id.clone(),
            vk: vk.to_owned(),
            inputs: prove_tasks.iter().map(from_prove_task).collect(),
            is_final,
            is_first_shard,
            is_leaf_layer: true,
            is_deferred,
            agg_index,
            ..Default::default()
        };

        if !is_deferred {
            agg_task.childs = prove_tasks
                .iter()
                .map(|t| Some(t.task_id.to_owned()))
                .collect();
        }

        agg_task
    }

    pub fn init_from_agg_tasks(agg_tasks: &[AggTask], agg_index: i32, is_final: bool) -> AggTask {
        let agg_task = AggTask {
            task_id: uuid::Uuid::new_v4().to_string(),
            state: TASK_STATE_UNPROCESSED,
            proof_id: agg_tasks[0].proof_id.clone(),
            inputs: agg_tasks.iter().map(|t| t.to_agg_input()).collect(),
            is_final,
            is_leaf_layer: false,
            agg_index,
            childs: agg_tasks
                .iter()
                .map(|t| Some(t.task_id.to_owned()))
                .collect(),
            ..Default::default()
        };
        agg_task
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_clear_child_task() {
        let left_task_id = "test_id_1";
        let right_task_id = "test_id_2";
        let mut agg_task = AggTask {
            state: TASK_STATE_UNPROCESSED,
            childs: vec![
                Some(left_task_id.to_string()),
                Some(right_task_id.to_string()),
            ],
            ..Default::default()
        };
        agg_task.clear_child_task(left_task_id);
        agg_task.clear_child_task(right_task_id);
        assert!(agg_task.childs[0].is_none());
        assert!(agg_task.childs[1].is_none());
    }
}

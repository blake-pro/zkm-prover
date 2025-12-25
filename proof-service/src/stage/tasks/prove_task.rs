use crate::proto::includes::v1::Program;
use crate::stage::tasks::Trace;
use serde_derive::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProveTask {
    pub task_id: String,
    pub program_id: String,
    pub proof_id: String,
    pub state: u32,
    pub base_dir: String,

    pub file_no: usize,
    pub is_deferred: bool,

    #[serde(skip_serializing, skip_deserializing)]
    // pub segment: Vec<u8>,
    pub segment: String,
    #[serde(skip_serializing, skip_deserializing, default = "default_program_arc")]
    pub program: Arc<Program>,

    #[serde(skip_serializing, skip_deserializing)]
    pub output: Vec<u8>, // output_receipt
    pub trace: Trace,
    // Number of times this task has failed
    pub failure_count: u32,
}

impl Default for ProveTask {
    fn default() -> Self {
        ProveTask {
            task_id: String::new(),
            program_id: String::new(),
            proof_id: String::new(),
            state: 0,
            base_dir: String::new(),
            file_no: 0,
            is_deferred: false,
            segment: String::new(),
            program: Arc::new(Program::default()),
            output: Vec::new(),
            trace: Trace::default(),
            failure_count: 0,
        }
    }
}

fn default_program_arc() -> Arc<Program> {
    Arc::new(Program::default())
}

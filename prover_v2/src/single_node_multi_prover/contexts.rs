use serde::{Deserialize, Serialize};
use zkm_core_executor::ExecutionRecord;
use zkm_prover::ZKMCircuitWitness;
use zkm_sdk::ZKMProof;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SplitContext {
    pub base_dir: String,
    pub program_id: String,
    pub elf_path: String,
    pub seg_size: u32,
    pub private_input_path: String,
    pub receipt_inputs_path: String,
}

impl SplitContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        basedir: &str,
        program_id: &str,
        elf_path: &str,
        seg_size: u32,
        private_input_path: &str,
        receipt_inputs_path: &str,
    ) -> Self {
        SplitContext {
            base_dir: basedir.to_string(),
            program_id: program_id.to_string(),
            elf_path: elf_path.to_string(),
            seg_size,
            private_input_path: private_input_path.to_string(),
            receipt_inputs_path: receipt_inputs_path.to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProveContext {
    pub program_id: String,
    pub elf_path: String,
    // execution record
    pub record: ExecutionRecord,
    pub seg_size: u32,
}

impl ProveContext {
    pub fn new(program_id: &str, elf_path: &str, record: ExecutionRecord, seg_size: u32) -> Self {
        ProveContext {
            program_id: program_id.to_string(),
            elf_path: elf_path.to_string(),
            record,
            seg_size,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AggContext(pub ZKMCircuitWitness);

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SnarkContext(pub ZKMProof::Compressed);

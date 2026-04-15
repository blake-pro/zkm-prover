use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SplitContext {
    pub base_dir: String,
    pub program_id: String,
    pub elf_path: String,
    pub seg_size: u32,
    pub seg_path: String,
    pub private_input_path: String,
    pub receipt_inputs_path: String,
}

impl SplitContext {
    pub fn new(
        basedir: &str,
        program_id: &str,
        elf_path: &str,
        seg_size: u32,
        seg_path: &str,
        private_input_path: &str,
        receipt_inputs_path: &str,
    ) -> Self {
        SplitContext {
            base_dir: basedir.to_string(),
            program_id: program_id.to_string(),
            elf_path: elf_path.to_string(),
            seg_size,
            seg_path: seg_path.to_string(),
            private_input_path: private_input_path.to_string(),
            receipt_inputs_path: receipt_inputs_path.to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProveContext {
    pub proof_id: String,
    pub program_id: String,
    pub index: usize,
    pub elf_path: String,
    // execution record
    // pub segment: Vec<u8>,
    pub segment: String,
    pub seg_size: u32,
    // pub receipts_input: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AggContext {
    // for leaf layer proof
    pub vk: Vec<u8>,
    // proofs for leaf layer, proofs and vks for other layers
    pub proofs: Vec<Vec<u8>>,
    pub is_complete: bool,
    // for leaf layer proof
    pub is_first_shard: bool,
    pub is_leaf_layer: bool,
    pub is_deferred: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SnarkContext {
    pub proof_id: String,
    pub agg_receipt: Vec<u8>,
    pub from_input: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SingleNodeContext {
    pub program_id: String,
    pub elf_path: String,
    pub base_dir: String,
    pub seg_size: u32,
    pub private_input_path: String,
    pub receipt_inputs_path: String,
    pub target_step: i32,
}

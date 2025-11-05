use super::super::contexts::SnarkContext;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SnarkTask {
    pub state: u32,
    pub proof_id: String,
    pub version: i32,

    pub input_dir: String,
    pub output_path: String,

    pub input: SnarkContext,

    pub output: Vec<u8>,
}

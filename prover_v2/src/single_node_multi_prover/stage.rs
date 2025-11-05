use super::tasks::{
    AggTask, ProveTask, SnarkTask, SplitTask, TASK_STATE_FAILED, TASK_STATE_INITIAL,
    TASK_STATE_PROCESSING, TASK_STATE_SUCCESS, TASK_STATE_UNPROCESSED,
};
use crate::contexts::SingleNodeContext;
use crate::proto::includes::v1::Step;
use rayon::prelude::*;
use std::{
    fmt::{Debug, Formatter},
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Default)]
pub struct Stage {
    pub ctx: SingleNodeContext,
    pub split_task: SplitTask,
    pub prove_tasks: Vec<ProveTask>,
    pub agg_tasks: Vec<AggTask>,
    pub snark_task: SnarkTask,
    pub is_error: bool,
    pub errmsg: String,
    pub step: Step,
    pub is_tasks_gen_done: bool,
}

macro_rules! on_task {
    ($src:ident, $dst:ident, $stage:ident) => {
        if $src.state == TASK_STATE_FAILED
            || $src.state == TASK_STATE_SUCCESS
            || $src.state == TASK_STATE_UNPROCESSED
        {
            $dst.state = $src.state;
            if TASK_STATE_UNPROCESSED != $src.state {
                // Fill in the output of the source task.
                $dst.output = $src.output.clone();
            }
            if TASK_STATE_FAILED == $src.state {
                $stage.is_error = true;
            }
        }
    };
}

macro_rules! get_task {
    ($src:ident) => {
        if $src.state == TASK_STATE_UNPROCESSED || $src.state == TASK_STATE_FAILED {
            $src.state = TASK_STATE_PROCESSING;
            return Some($src.clone());
        }
        return None
    };
}

impl Stage {
    pub fn new(ctx: SingleNodeContext) -> Self {
        Stage {
            ctx,
            step: Step::Init,
            split_task: SplitTask::default(),
            prove_tasks: Vec::new(),
            agg_tasks: Vec::new(),
            snark_task: SnarkTask::default(),
            is_error: false,
            errmsg: "".to_string(),
            is_tasks_gen_done: false,
        }
    }

    // todo: remove
    pub fn dispatch(&mut self) {
        match self.step {
            Step::Init => {
                self.gen_split_task();
                self.step = Step::Prove;
            }
            Step::Prove => {
                self.gen_prove_task();
                tracing::debug!("generate {} tasks", self.prove_tasks.len());
                if self.split_task.state == TASK_STATE_SUCCESS && !self.is_tasks_gen_done {
                    self.gen_prove_task_post();
                    tracing::info!(
                        "Split done. Generate {} prove_tasks",
                        self.prove_tasks.len()
                    );
                    self.gen_agg_tasks();

                    self.is_tasks_gen_done = true;
                    // clear agg tasks' child task
                    let successful_task_ids = self
                        .prove_tasks
                        .iter()
                        .filter(|t| t.state == TASK_STATE_SUCCESS)
                        .map(|t| t.task_id.clone())
                        .collect::<Vec<_>>();
                    for id in successful_task_ids {
                        self.clear_agg_child_task(&id);
                    }
                }

                if self.is_tasks_gen_done
                    && self
                        .prove_tasks
                        .iter()
                        .all(|task| task.state == TASK_STATE_SUCCESS)
                    && self
                        .agg_tasks
                        .iter()
                        .all(|task| task.state == TASK_STATE_SUCCESS)
                {
                    if self.ctx.target_step == Step::Agg {
                        self.step = Step::End;
                    } else {
                        self.gen_snark_task();
                        self.step = Step::Snark;
                    }
                }
            }
            Step::Snark => {
                if self.snark_task.state == TASK_STATE_SUCCESS {
                    self.step = Step::End;
                }
            }
            _ => {}
        }
    }

    pub fn is_success(&self) -> bool {
        if self.step == Step::End || self.snark_task.state == TASK_STATE_SUCCESS {
            return true;
        }
        false
    }

    pub fn is_error(&self) -> bool {
        self.is_error
    }

    fn gen_split_task(&mut self) {
        assert_eq!(self.split_task.state, TASK_STATE_INITIAL);
        self.split_task.state = TASK_STATE_UNPROCESSED;
        self.split_task.ctx.base_dir.clone_from(&self.ctx.base_dir);
        self.split_task.ctx.elf_path.clone_from(&self.ctx.elf_path);
        self.split_task
            .ctx
            .program_id
            .clone_from(&self.ctx.program_id);
        self.split_task
            .ctx
            .private_input_path
            .clone_from(&self.ctx.private_input_path);
        self.split_task
            .ctx
            .receipt_inputs_path
            .clone_from(&self.ctx.receipt_inputs_path);
        self.split_task.ctx.seg_size = self.ctx.seg_size;
    }

    pub fn get_split_task(&mut self) -> Option<SplitTask> {
        let src = &mut self.split_task;
        get_task!(src);
    }

    pub fn on_split_task(&mut self, split_task: &mut SplitTask) {
        let dst = &mut self.split_task;
        dst.total_steps = split_task.total_steps;
        dst.total_segments = split_task.total_segments;
        on_task!(split_task, dst, self);
    }

    fn gen_prove_task(&mut self) {
        if self.is_tasks_gen_done {
            return;
        }
        // Pre-allocate 64 tasks
        if self.prove_tasks.is_empty() {
            self.prove_tasks = (0..16)
                .into_par_iter()
                .map(|i| self.task_with_no(i))
                .collect();
        }
        let file_numbers: usize = match std::fs::read_to_string(format!(
            "{}/segments.txt",
            self.generate_task.seg_path
        )) {
            Ok(content) => match content.trim().parse() {
                Ok(n) => n,
                Err(_) => return,
            },
            Err(_) => return,
        };

        // generate prove tasks
        for file_no in self.prove_tasks.len()..file_numbers {
            let task = self.task_with_no(file_no);
            self.prove_tasks.push(task);
            tracing::debug!("insert {file_no}");
        }
    }

    fn gen_prove_task_post(&mut self) {
        // ensure all the prove tasks are generated
        {
            if self.prove_tasks.len() > self.split_task.total_segments as usize {
                self.prove_tasks
                    .truncate(self.split_task.total_segments as usize)
            } else {
                let missing_tasks = (self.prove_tasks.len()
                    ..self.split_task.total_segments as usize)
                    .into_par_iter()
                    .map(|i| self.task_with_no(i))
                    .collect::<Vec<_>>();
                self.prove_tasks.extend_from_slice(&missing_tasks);
            }
        }

        {
            let files = common::file::new(&self.generate_task.seg_path)
                .read_dir()
                .unwrap();
            let mut deferred_files: Vec<(usize, String)> = Vec::new();
            for file_name in files {
                if let Some(name) = file_name.strip_prefix("deferred_proof_") {
                    if let Ok(num) = name.parse::<usize>() {
                        deferred_files.push((num, file_name));
                    }
                }
            }
            deferred_files.sort_by_key(|(n, _)| *n);
            tracing::info!("Generate {} deferred proofs", deferred_files.len());

            for (file_no, file_name) in deferred_files.into_iter() {
                let prove_task = ProveTask {
                    task_id: uuid::Uuid::new_v4().to_string(),
                    proof_id: self.generate_task.proof_id.clone(),
                    state: TASK_STATE_SUCCESS,
                    base_dir: self.generate_task.base_dir.clone(),
                    file_no,
                    is_deferred: true,
                    program: self.generate_task.gen_program(),
                    output: safe_read(&format!("{}/{file_name}", self.generate_task.seg_path)),
                    ..Default::default()
                };
                self.prove_tasks.push(prove_task);
            }
        }
    }

    pub fn get_prove_task(&mut self) -> Option<ProveTask> {
        for prove_task in self.prove_tasks.iter_mut() {
            if prove_task.state == TASK_STATE_UNPROCESSED || prove_task.state == TASK_STATE_FAILED {
                prove_task.state = TASK_STATE_PROCESSING;
                return Some(prove_task.clone());
            }
        }
        None
    }

    pub fn on_prove_task(&mut self, prove_task: &mut ProveTask) {
        for item_task in self.prove_tasks.iter_mut() {
            if item_task.task_id == prove_task.task_id && item_task.state == TASK_STATE_PROCESSING {
                on_task!(prove_task, item_task, self);
                break;
            }
        }
        // clear agg‘s child task
        if prove_task.state == TASK_STATE_SUCCESS {
            self.clear_agg_child_task(&prove_task.task_id);
        }
    }

    // caller guarantees prove_task is done.
    fn clear_agg_child_task(&mut self, task_id: &str) {
        for agg_task in &mut self.agg_tasks {
            if agg_task.clear_child_task(task_id) {
                break;
            }
        }
    }

    pub fn count_unfinished_prove_tasks(&self) -> usize {
        self.prove_tasks
            .iter()
            .filter(|task| task.state != TASK_STATE_SUCCESS)
            .count()
    }

    pub fn count_processing_prove_tasks(&self) -> usize {
        self.prove_tasks
            .iter()
            .filter(|task| task.state == TASK_STATE_PROCESSING)
            .count()
    }

    pub fn gen_agg_tasks(&mut self) {
        use prover_v2::FIRST_LAYER_BATCH_SIZE;
        // The batch size for reducing two layers of recursion.
        let batch_size = 2;
        // The batch size for reducing the first layer of recursion.
        let first_layer_batch_size = FIRST_LAYER_BATCH_SIZE;

        let mut agg_index = 0;
        let mut result = Vec::new();
        // process the first layer
        let vk = common::file::new(&format!("{}/vk.bin", self.generate_task.base_dir))
            .read()
            .expect("read vk");

        let (normal_prove_tasks, deferred_prove_tasks): (Vec<_>, Vec<_>) = self
            .prove_tasks
            .iter()
            .cloned()
            .partition(|task| !task.is_deferred);
        let is_complete = normal_prove_tasks.len() == 1 && deferred_prove_tasks.is_empty();

        for (batch_index, batch) in normal_prove_tasks
            .chunks(first_layer_batch_size)
            .enumerate()
        {
            let agg_task = AggTask::init_from_prove_tasks(
                &vk,
                batch,
                agg_index,
                is_complete,
                batch_index == 0,
                false,
            );
            result.push(agg_task);
            agg_index += 1;
        }
        // already batched during the split phase
        for batch in deferred_prove_tasks {
            let agg_task =
                AggTask::init_from_prove_tasks(&vk, &[batch], agg_index, is_complete, false, true);
            result.push(agg_task);
            agg_index += 1;
        }
        self.agg_tasks.append(&mut result.clone());

        let mut current_length = result.len();
        while current_length > 1 {
            let mut new_result = Vec::new();
            for batch in result.chunks(batch_size) {
                let agg_task = AggTask::init_from_agg_tasks(batch, agg_index, false);
                self.agg_tasks.push(agg_task.clone());
                new_result.push(agg_task);
                agg_index += 1;
            }

            result = new_result;
            current_length = result.len();
        }

        if let Some(last) = self.agg_tasks.last_mut() {
            last.is_final = true;
        }
    }

    pub fn get_agg_task(&mut self) -> Option<AggTask> {
        let mut result: Option<AggTask> = None;
        for agg_task in &mut self.agg_tasks {
            if agg_task.childs.iter().any(|c| c.is_some()) {
                tracing::debug!("Skipping agg_task: childs: {:?}", agg_task.childs);
                continue;
            }
            if agg_task.state == TASK_STATE_UNPROCESSED || agg_task.state == TASK_STATE_FAILED {
                agg_task.state = TASK_STATE_PROCESSING;
                result = Some(agg_task.clone());
                break;
            }
        }
        // Fill in the inputs
        if let Some(agg_task) = &mut result {
            agg_task.inputs.iter_mut().for_each(|input| {
                if input.is_agg {
                    let tmp = self
                        .agg_tasks
                        .iter()
                        .find(|x| x.task_id == input.computed_request_id)
                        .unwrap();
                    input.receipt_input = tmp.output.clone();
                } else {
                    let tmp = self
                        .prove_tasks
                        .iter()
                        .find(|x| x.task_id == input.computed_request_id)
                        .unwrap();
                    input.receipt_input = tmp.output.clone();
                }
            });
        };
        result
    }

    pub fn on_agg_task(&mut self, agg_task: &mut AggTask) {
        for item_task in &mut self.agg_tasks {
            if item_task.task_id == agg_task.task_id && item_task.state == TASK_STATE_PROCESSING {
                on_task!(agg_task, item_task, self);
                break;
            }
        }
        if agg_task.state == TASK_STATE_SUCCESS {
            for item_task in &mut self.agg_tasks {
                if item_task.clear_child_task(&agg_task.task_id) {
                    break;
                }
            }

            // write final agg task output
            if agg_task.is_final && self.generate_task.target_step == Step::Agg {
                // Here we also use snark_path to store agg proof ;
                let mut f = std::fs::File::create(&self.generate_task.snark_path)
                    .unwrap_or_else(|_| panic!("can not open {}", &self.generate_task.snark_path));
                f.write_all(&agg_task.output).unwrap();
            }
        }
    }

    pub fn gen_snark_task(&mut self) {
        assert_eq!(self.snark_task.state, TASK_STATE_INITIAL);
        self.snark_task
            .proof_id
            .clone_from(&self.generate_task.proof_id.clone());
        self.snark_task
            .input_dir
            .clone_from(&self.generate_task.agg_path);
        self.snark_task
            .output_path
            .clone_from(&self.generate_task.snark_path);
        self.snark_task.task_id = uuid::Uuid::new_v4().to_string();
        self.snark_task.state = TASK_STATE_UNPROCESSED;
        // fill in the input receipts
        if self.generate_task.from_step == Step::Init {
            for agg_task in &self.agg_tasks {
                if agg_task.is_final {
                    self.snark_task.agg_receipt = agg_task.output.clone();
                    self.snark_task.from_input = false;
                }
            }
        } else if self.generate_task.from_step == Step::Agg {
            // read from receipt_inputs_path
            let receipt_datas = std::fs::read(&self.generate_task.receipt_inputs_path).unwrap();
            let receipts = bincode::deserialize::<Vec<Vec<u8>>>(&receipt_datas).unwrap();
            self.snark_task.agg_receipt = receipts[0].clone();
            self.snark_task.from_input = true;
        } else {
            unreachable!(
                "gen_snark_task: unsupported from_step: {:?}",
                self.generate_task.from_step
            );
        }

        tracing::info!(
            "gen_snark_task: {:?} {:?}",
            self.snark_task.proof_id,
            self.snark_task.task_id
        );
    }

    pub fn get_snark_task(&mut self) -> Option<SnarkTask> {
        let src = &mut self.snark_task;
        tracing::debug!(
            "get_snark_task: proof_id:task_id: {:?}:{:?} => status:{}",
            src.proof_id,
            src.task_id,
            src.state
        );
        get_task!(src);
    }

    pub fn on_snark_task(&mut self, snark_task: &mut SnarkTask) {
        let dst = &mut self.snark_task;
        // write snark proof to disk
        // TODO: handle the result gracefully
        let mut f = std::fs::File::create(&self.generate_task.snark_path)
            .unwrap_or_else(|_| panic!("can not open {}", &self.generate_task.snark_path));
        f.write_all(&snark_task.output).unwrap();
        on_task!(snark_task, dst, self);
    }
}

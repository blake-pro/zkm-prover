mod agg_prover;
mod contexts;
mod executor;
mod root_prover;
mod snark_prover;
mod stage;
mod tasks;

use crate::contexts::SingleNodeContext;
use crate::snark_prover::SnarkProver;
use crate::{get_prover, NetworkProve, KEY_CACHE, PROGRAM_CACHE};
use common::file;
use std::path::PathBuf;
use tokio::sync::mpsc;
use zkm_core_executor::ZKMReduceProof;
use zkm_prover::ZKMVerifyingKey;
use zkm_sdk::network::prover::stage_service::Step;
use zkm_sdk::ZKMProof;
use zkm_stark::koala_bear_poseidon2::KoalaBearPoseidon2;
use zkm_stark::{MachineProver, StarkVerifyingKey};

#[derive(Default)]
pub struct SingleNodeMultiProver {
    proving_key_paths: String,
}

impl SingleNodeMultiProver {
    pub fn new(proving_key_paths: &str) -> Self {
        Self {
            proving_key_paths: proving_key_paths.into(),
        }
    }
    pub fn run(&self, ctx: &SingleNodeContext) -> anyhow::Result<()> {
        // spawn one thread to run Split

        //

        // Distributed (multi-node) handler
        let (tx, mut rx) = mpsc::channel(128);
        stage.dispatch();

        let mut interval = time::interval(time::Duration::from_millis(200));
        loop {
            let current_step = stage.step;
            match stage.step {
                Step::Prove => {
                    // Dispatch split tasks.
                    if let Some(task_payload) = stage.get_split_task() {
                        dispatch_task(
                            task_payload,
                            prover_client::split,
                            Task::Split,
                            tx.clone(),
                            tls_config.clone(),
                            cur_prover_num.clone(),
                            max_prover_num,
                        );
                    }

                    // Dispatch prove tasks until the concurrent prover limit is reached.
                    while stage.count_processing_prove_tasks() < max_prover_num as usize {
                        if let Some(task_payload) = stage.get_prove_task() {
                            dispatch_task(
                                task_payload,
                                prover_client::prove,
                                Task::Prove,
                                tx.clone(),
                                tls_config.clone(),
                                cur_prover_num.clone(),
                                max_prover_num,
                            );
                        } else {
                            // No more prove tasks available, break the inner loop.
                            break;
                        }
                    }

                    // Dispatch aggregate tasks if conditions are met.
                    while stage.is_tasks_gen_done
                        && stage.count_unfinished_prove_tasks() < max_prover_num as usize
                    {
                        if let Some(task_payload) = stage.get_agg_task() {
                            tracing::debug!("get_agg_task: true");
                            dispatch_task(
                                task_payload,
                                prover_client::aggregate,
                                Task::Agg,
                                tx.clone(),
                                tls_config.clone(),
                                cur_prover_num.clone(),
                                max_prover_num,
                            );
                        } else {
                            // No more aggregation tasks available, break the inner loop.
                            tracing::debug!("get_agg_task: false");
                            break;
                        }
                    }
                }
                Step::Snark => {
                    if let Some(task_payload) = stage.get_snark_task() {
                        dispatch_task(
                            task_payload,
                            prover_client::snark_proof,
                            Task::Snark,
                            tx.clone(),
                            tls_config.clone(),
                            cur_prover_num.clone(),
                            max_prover_num,
                        );
                    }
                }
                _ => {}
            }

            tokio::select! {
                task = rx.recv() => {
                    if let Some(task) = task {
                        match task {
                            Task::Split(mut data) => {
                                stage.on_split_task(&mut data);
                                save_task!(data, db, TASK_ITYPE_SPLIT);
                            },
                            Task::Prove(mut data) => {
                                stage.on_prove_task(&mut data);
                                // save_task!(data, db, TASK_ITYPE_PROVE);
                            },
                            Task::Agg(mut data) => {
                                stage.on_agg_task(&mut data);
                                // save_task!(data, db, TASK_ITYPE_AGG);
                            },
                            Task::Snark(mut data) => {
                                stage.on_snark_task(&mut data);
                                save_task!(data, db, TASK_ITYPE_FINAL);
                            },
                        };
                    }
                },
                _ = interval.tick() => {
                }
            }
            if stage.is_success() || stage.is_error() {
                break;
            }
            stage.dispatch();
        }

        let result = if stage.is_success() && generate_context.target_step == Step::Snark {
            file::new(&generate_context.snark_path)
                .read()
                .unwrap_or_default()
        } else {
            vec![]
        };
        finalize_stage_task(&task, &stage, task_start_time, result, &db).await;
        Ok(())
    }
}

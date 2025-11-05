use super::contexts::ProveContext;
use crate::{get_prover, NetworkProve, Segment, KEY_CACHE, PROGRAM_CACHE};

use common::file;
use zkm_core_machine::utils::trace_checkpoint;
use zkm_prover::CoreSC;
use zkm_stark::{MachineProver, MachineProvingKey, ShardProof, StarkGenericConfig};

#[derive(Default)]
pub struct RootProver {}

impl RootProver {
    pub fn prove(&self, ctx: &mut ProveContext) -> anyhow::Result<ShardProof<CoreSC>> {
        let network_prove = NetworkProve::new(ctx.seg_size);
        let opts = network_prove.opts.core_opts;
        // todo: use new prover
        let prover = get_prover();

        let mut cache = KEY_CACHE.lock();
        let (pk, _) = cache
            .cache
            .get(&ctx.program_id)
            .expect("Proving key not found");

        let now = std::time::Instant::now();
        prover.core_prover.machine().generate_dependencies(
            std::slice::from_mut(&mut ctx.record),
            &opts,
            None,
        );
        tracing::info!("generate dependencies time: {:?}", now.elapsed());

        // Fix the shape of the record.
        let now = std::time::Instant::now();
        if let Some(shape_config) = &prover.core_shape_config {
            shape_config.fix_shape(&mut ctx.record)?;
        }
        tracing::info!("fix shape time: {:?}", now.elapsed());
        let now = std::time::Instant::now();
        let main_trace = prover.core_prover.generate_traces(&ctx.record);
        tracing::info!("generate traces time: {:?}", now.elapsed());

        let mut challenger = prover.core_prover.config().challenger();
        pk.observe_into(&mut challenger);
        let now = std::time::Instant::now();
        let main_data = prover.core_prover.commit(&ctx.record, main_trace);
        tracing::info!("commit time: {:?}", now.elapsed());
        let now = std::time::Instant::now();
        let proof = prover.core_prover.open(pk, main_data, &mut challenger)?;
        tracing::info!("open time: {:?}", now.elapsed());

        Ok(proof)
    }
}

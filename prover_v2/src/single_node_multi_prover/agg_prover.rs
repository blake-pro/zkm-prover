use super::contexts::AggContext;
use crate::{get_prover, NetworkProve, ProverComponents};
use zkm_core_executor::ZKMReduceProof;
use zkm_prover::build::Witnessable;
use zkm_prover::{InnerSC, ZKMCircuitWitness, ZKMProver, ZKMRecursionProverError};
use zkm_recursion_circuit::machine::{
    ZKMCompressWitnessValues, ZKMDeferredWitnessValues, ZKMRecursionWitnessValues,
};
use zkm_recursion_compiler::config::InnerConfig;
use zkm_recursion_core::Runtime;
use zkm_sdk::ZKMProof;
use zkm_stark::{
    Challenge, MachineProver, MachineProvingKey, StarkGenericConfig, Val, ZKMCoreOpts,
};

#[derive(Default)]
pub struct AggProver {}

impl AggProver {
    pub fn prove(&self, ctx: &AggContext) -> anyhow::Result<ZKMProof::Compressed> {
        let network_prove = NetworkProve::default();
        // todo: use new prover
        let prover = get_prover();

        let (program, witness_stream) = tracing::debug_span!("get program and witness stream")
            .in_scope(|| match ctx.0 {
                ZKMCircuitWitness::Core(input) => {
                    let mut witness_stream = Vec::new();
                    Witnessable::<InnerConfig>::write(&input, &mut witness_stream);
                    (prover.recursion_program(&input), witness_stream)
                }
                ZKMCircuitWitness::Deferred(input) => {
                    let mut witness_stream = Vec::new();
                    Witnessable::<InnerConfig>::write(&input, &mut witness_stream);
                    (prover.deferred_program(&input), witness_stream)
                }
                ZKMCircuitWitness::Compress(input) => {
                    let mut witness_stream = Vec::new();

                    let input_with_merkle = prover.make_merkle_proofs(input);

                    Witnessable::<InnerConfig>::write(&input_with_merkle, &mut witness_stream);

                    (prover.compress_program(&input_with_merkle), witness_stream)
                }
            });

        // Execute the runtime.
        let record = tracing::debug_span!("execute runtime").in_scope(|| {
            let mut runtime = Runtime::<Val<InnerSC>, Challenge<InnerSC>, _>::new(
                program.clone(),
                prover.compress_prover.config().perm.clone(),
            );
            runtime.witness_stream = witness_stream.into();
            runtime
                .run()
                .map(|_| runtime.record)
                .map_err(|e| ZKMRecursionProverError::RuntimeError(e.to_string()))
        })?;

        // Generate the dependencies.
        let mut records = vec![record];
        tracing::debug_span!("generate dependencies").in_scope(|| {
            prover.compress_prover.machine().generate_dependencies(
                &mut records,
                &network_prove.opts.recursion_opts,
                None,
            )
        });

        // Generate the traces.
        let record = records.into_iter().next().unwrap();
        let traces = tracing::debug_span!("generate traces")
            .in_scope(|| prover.compress_prover.generate_traces(&record));

        let (vk, proof) = tracing::debug_span!("batch").in_scope(|| {
            // Get the keys.
            let (pk, vk) = tracing::debug_span!("Setup compress program")
                .in_scope(|| prover.compress_prover.setup(&program));

            // Observe the proving key.
            let mut challenger = prover.compress_prover.config().challenger();
            tracing::debug_span!("observe proving key").in_scope(|| {
                pk.observe_into(&mut challenger);
            });

            // Commit to the record and traces.
            let data = tracing::debug_span!("commit")
                .in_scope(|| prover.compress_prover.commit(&record, traces));

            // Generate the proof.
            let proof = tracing::debug_span!("open").in_scope(|| {
                prover
                    .compress_prover
                    .open(&pk, data, &mut challenger)
                    .unwrap()
            });

            // Verify the proof.
            #[cfg(feature = "debug")]
            prover
                .compress_prover
                .machine()
                .verify(
                    &vk,
                    &zkm2_stark::MachineProof {
                        shard_proofs: vec![proof.clone()],
                    },
                    &mut prover.compress_prover.config().challenger(),
                )
                .unwrap();

            (vk, proof)
        });

        Ok(ZKMProof::Compressed(Box::new(ZKMReduceProof { vk, proof })))
    }
}

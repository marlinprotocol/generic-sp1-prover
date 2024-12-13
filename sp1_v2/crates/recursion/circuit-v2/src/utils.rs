use std::borrow::BorrowMut;

use p3_baby_bear::BabyBear;
use p3_bn254_fr::Bn254Fr;
use p3_field::AbstractField;
use p3_field::PrimeField32;

use sp1_recursion_compiler::{
    circuit::CircuitV2Builder,
    ir::{Builder, Config, Felt, Var},
};
use sp1_recursion_core_v2::{
    air::{RecursionPublicValues, NUM_PV_ELMS_TO_HASH, RECURSIVE_PROOF_NUM_PV_ELTS},
    DIGEST_SIZE,
};
use sp1_stark::Word;

/// Register and commits the recursion public values.
pub fn commit_recursion_public_values<C: Config>(
    builder: &mut Builder<C>,
    public_values: &RecursionPublicValues<Felt<C::F>>,
) {
    let mut pv_elements: [Felt<_>; RECURSIVE_PROOF_NUM_PV_ELTS] =
        core::array::from_fn(|_| builder.uninit());
    *pv_elements.as_mut_slice().borrow_mut() = *public_values;
    let pv_elms_no_digest = &pv_elements[0..NUM_PV_ELMS_TO_HASH];

    for value in pv_elms_no_digest.iter() {
        builder.register_public_value(*value);
    }

    // Hash the public values.
    let pv_digest = builder.poseidon2_hash_v2(&pv_elements[0..NUM_PV_ELMS_TO_HASH]);
    for element in pv_digest {
        builder.commit_public_value(element);
    }
}

/// Convert 8 BabyBear words into a Bn254Fr field element by shifting by 31 bits each time. The last
/// word becomes the least significant bits.
#[allow(dead_code)]
pub fn babybears_to_bn254(digest: &[BabyBear; 8]) -> Bn254Fr {
    let mut result = Bn254Fr::zero();
    for word in digest.iter() {
        // Since BabyBear prime is less than 2^31, we can shift by 31 bits each time and still be
        // within the Bn254Fr field, so we don't have to truncate the top 3 bits.
        result *= Bn254Fr::from_canonical_u64(1 << 31);
        result += Bn254Fr::from_canonical_u32(word.as_canonical_u32());
    }
    result
}

/// Convert 32 BabyBear bytes into a Bn254Fr field element. The first byte's most significant 3 bits
/// (which would become the 3 most significant bits) are truncated.
#[allow(dead_code)]
pub fn babybear_bytes_to_bn254(bytes: &[BabyBear; 32]) -> Bn254Fr {
    let mut result = Bn254Fr::zero();
    for (i, byte) in bytes.iter().enumerate() {
        debug_assert!(byte < &BabyBear::from_canonical_u32(256));
        if i == 0 {
            // 32 bytes is more than Bn254 prime, so we need to truncate the top 3 bits.
            result = Bn254Fr::from_canonical_u32(byte.as_canonical_u32() & 0x1f);
        } else {
            result *= Bn254Fr::from_canonical_u32(256);
            result += Bn254Fr::from_canonical_u32(byte.as_canonical_u32());
        }
    }
    result
}

pub fn felts_to_bn254_var<C: Config>(
    builder: &mut Builder<C>,
    digest: &[Felt<C::F>; DIGEST_SIZE],
) -> Var<C::N> {
    let var_2_31: Var<_> = builder.constant(C::N::from_canonical_u32(1 << 31));
    let result = builder.constant(C::N::zero());
    for (i, word) in digest.iter().enumerate() {
        let word_bits = builder.num2bits_f_circuit(*word);
        let word_var = builder.bits2num_v_circuit(&word_bits);
        if i == 0 {
            builder.assign(result, word_var);
        } else {
            builder.assign(result, result * var_2_31 + word_var);
        }
    }
    result
}

pub fn felt_bytes_to_bn254_var<C: Config>(
    builder: &mut Builder<C>,
    bytes: &[Felt<C::F>; 32],
) -> Var<C::N> {
    let var_256: Var<_> = builder.constant(C::N::from_canonical_u32(256));
    let zero_var: Var<_> = builder.constant(C::N::zero());
    let result = builder.constant(C::N::zero());
    for (i, byte) in bytes.iter().enumerate() {
        let byte_bits = builder.num2bits_f_circuit(*byte);
        if i == 0 {
            // Since 32 bytes doesn't fit into Bn254, we need to truncate the top 3 bits.
            // For first byte, zero out 3 most significant bits.
            for i in 0..3 {
                builder.assign(byte_bits[8 - i - 1], zero_var);
            }
            let byte_var = builder.bits2num_v_circuit(&byte_bits);
            builder.assign(result, byte_var);
        } else {
            let byte_var = builder.bits2num_v_circuit(&byte_bits);
            builder.assign(result, result * var_256 + byte_var);
        }
    }
    result
}

pub fn words_to_bytes<T: Copy>(words: &[Word<T>]) -> Vec<T> {
    words.iter().flat_map(|w| w.0).collect::<Vec<_>>()
}

#[cfg(any(test, feature = "export-tests"))]
pub(crate) mod tests {
    use std::sync::Arc;

    use sp1_core_machine::utils::{run_test_machine_with_prover, setup_logger};
    use sp1_recursion_compiler::{asm::AsmConfig, circuit::AsmCompiler, ir::DslIr};

    use sp1_recursion_compiler::ir::TracedVec;
    use sp1_recursion_core_v2::{machine::RecursionAir, Runtime};
    use sp1_stark::{
        baby_bear_poseidon2::BabyBearPoseidon2, CpuProver, InnerChallenge, InnerVal, MachineProver,
    };

    use crate::witness::WitnessBlock;

    type SC = BabyBearPoseidon2;
    type F = InnerVal;
    type EF = InnerChallenge;

    /// A simplified version of some code from `recursion/core/src/stark/mod.rs`.
    /// Takes in a program and runs it with the given witness and generates a proof with a variety
    /// of machines depending on the provided test_config.
    pub(crate) fn run_test_recursion_with_prover<P: MachineProver<SC, RecursionAir<F, 3, 0>>>(
        operations: TracedVec<DslIr<AsmConfig<F, EF>>>,
        witness_stream: impl IntoIterator<Item = WitnessBlock<AsmConfig<F, EF>>>,
    ) {
        setup_logger();

        let compile_span = tracing::debug_span!("compile").entered();
        let mut compiler = AsmCompiler::<AsmConfig<F, EF>>::default();
        let program = Arc::new(compiler.compile(operations));
        compile_span.exit();

        let config = SC::default();

        let run_span = tracing::debug_span!("run the recursive program").entered();
        let mut runtime = Runtime::<F, EF, _>::new(program.clone(), config.perm.clone());
        runtime.witness_stream.extend(witness_stream);
        tracing::debug_span!("run").in_scope(|| runtime.run().unwrap());
        assert!(runtime.witness_stream.is_empty());
        run_span.exit();

        let records = vec![runtime.record];

        // Run with the poseidon2 wide chip.
        let proof_wide_span = tracing::debug_span!("Run test with wide machine").entered();
        let wide_machine = RecursionAir::<_, 3, 0>::machine_wide(SC::default());
        let (pk, vk) = wide_machine.setup(&program);
        let result = run_test_machine_with_prover::<_, _, P>(records.clone(), wide_machine, pk, vk);
        proof_wide_span.exit();

        if let Err(e) = result {
            panic!("Verification failed: {:?}", e);
        }
    }

    #[allow(dead_code)]
    pub(crate) fn run_test_recursion(
        operations: TracedVec<DslIr<AsmConfig<F, EF>>>,
        witness_stream: impl IntoIterator<Item = WitnessBlock<AsmConfig<F, EF>>>,
    ) {
        run_test_recursion_with_prover::<CpuProver<_, _>>(operations, witness_stream)
    }
}

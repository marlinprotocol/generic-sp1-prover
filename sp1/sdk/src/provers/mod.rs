mod local;
mod mock;

use crate::{SP1CompressedProof, SP1PlonkBn254Proof, SP1Proof};
use anyhow::Result;
pub use local::LocalProver;
pub use mock::MockProver;
use sp1_core::stark::MachineVerificationError;
use sp1_core::SP1_CIRCUIT_VERSION;
use sp1_prover::CoreSC;
use sp1_prover::InnerSC;
use sp1_prover::SP1CoreProofData;
use sp1_prover::SP1Prover;
use sp1_prover::SP1ReduceProof;
use sp1_prover::{SP1ProvingKey, SP1Stdin, SP1VerifyingKey};
use strum_macros::EnumString;
use thiserror::Error;

/// The type of prover.
#[derive(Debug, PartialEq, EnumString)]
pub enum ProverType {
    Local,
    Mock,
    Network,
}

#[derive(Error, Debug)]
pub enum SP1VerificationError {
    #[error("Version mismatch")]
    VersionMismatch(String),
    #[error("Core machine verification error: {0}")]
    Core(MachineVerificationError<CoreSC>),
    #[error("Recursion verification error: {0}")]
    Recursion(MachineVerificationError<InnerSC>),
    #[error("Plonk verification error: {0}")]
    Plonk(anyhow::Error),
}

/// An implementation of [crate::ProverClient].
pub trait Prover: Send + Sync {
    fn id(&self) -> ProverType;

    fn sp1_prover(&self) -> &SP1Prover;

    fn version(&self) -> &str {
        SP1_CIRCUIT_VERSION
    }

    fn setup(&self, elf: &[u8]) -> (SP1ProvingKey, SP1VerifyingKey);

    /// Prove the execution of a RISCV ELF with the given inputs.
    fn prove(&self, pk: &SP1ProvingKey, stdin: SP1Stdin) -> Result<SP1Proof>;

    /// Generate a compressed proof of the execution of a RISCV ELF with the given inputs.
    fn prove_compressed(&self, pk: &SP1ProvingKey, stdin: SP1Stdin) -> Result<SP1CompressedProof>;

    /// Given an SP1 program and input, generate a PLONK proof that can be verified on-chain.
    fn prove_plonk(&self, pk: &SP1ProvingKey, stdin: SP1Stdin) -> Result<SP1PlonkBn254Proof>;

    /// Verify that an SP1 proof is valid given its vkey and metadata.
    fn verify(&self, proof: &SP1Proof, vkey: &SP1VerifyingKey) -> Result<(), SP1VerificationError> {
        if proof.sp1_version != self.version() {
            return Err(SP1VerificationError::VersionMismatch(
                proof.sp1_version.clone(),
            ));
        }
        self.sp1_prover()
            .verify(&SP1CoreProofData(proof.proof.clone()), vkey)
            .map_err(SP1VerificationError::Core)
    }

    /// Verify that a compressed SP1 proof is valid given its vkey and metadata.
    fn verify_compressed(
        &self,
        proof: &SP1CompressedProof,
        vkey: &SP1VerifyingKey,
    ) -> Result<(), SP1VerificationError> {
        if proof.sp1_version != self.version() {
            return Err(SP1VerificationError::VersionMismatch(
                proof.sp1_version.clone(),
            ));
        }
        self.sp1_prover()
            .verify_compressed(
                &SP1ReduceProof {
                    proof: proof.proof.clone(),
                },
                vkey,
            )
            .map_err(SP1VerificationError::Recursion)
    }

    /// Verify that a SP1 PLONK proof is valid. Verify that the public inputs of the PlonkBn254 proof match
    /// the hash of the VK and the committed public values of the SP1ProofWithPublicValues.
    fn verify_plonk(
        &self,
        proof: &SP1PlonkBn254Proof,
        vkey: &SP1VerifyingKey,
    ) -> Result<(), SP1VerificationError> {
        if proof.sp1_version != self.version() {
            return Err(SP1VerificationError::VersionMismatch(
                proof.sp1_version.clone(),
            ));
        }
        let sp1_prover = self.sp1_prover();

        let plonk_bn254_aritfacts = if sp1_prover::build::sp1_dev_mode() {
            sp1_prover::build::plonk_bn254_artifacts_dev_dir()
        } else {
            sp1_prover::build::try_install_plonk_bn254_artifacts()
        };
        sp1_prover
            .verify_plonk_bn254(
                &proof.proof,
                vkey,
                &proof.public_values,
                &plonk_bn254_aritfacts,
            )
            .map_err(SP1VerificationError::Plonk)?;

        Ok(())
    }
}

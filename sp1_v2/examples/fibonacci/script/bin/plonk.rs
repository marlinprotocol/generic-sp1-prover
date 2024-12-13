


use sp1_sdk::{utils, ProverClient, SP1Stdin};

/// The ELF we want to execute inside the zkVM.
// const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

fn main() {
    // Setup logging.
    // utils::setup_logger();

    // // Create an input stream and write '500' to it.
    // let n = 10u32;

    // let mut stdin = SP1Stdin::new();
    // stdin.write(&n);
    utils::setup_logger();
    let client = ProverClient::local();
    let elf =
        include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");
    let (pk, vk) = client.setup(elf);
    let mut stdin = SP1Stdin::new();
    stdin.write(&10usize);

    // Generate proof & verify.
    let mut proof = client.prove(&pk, stdin).plonk().run().unwrap();
    client.verify(&proof, &vk).unwrap();

    // Test invalid public values.
    // proof.public_values = SP1PublicValues::from(&[255, 4, 84]);
    // if client.verify(&proof, &vk).is_ok() {
    //     panic!("verified proof with invalid public values")
    // }
    // Generate the proof for the given program and input.
    // let client = ProverClient::new();
    // let (pk, vk) = client.setup(ELF);
    // let proof = client.prove(&pk, stdin).groth16().run().unwrap();

    // println!("generated proof");

    // // Get the public values as bytes.
    // let public_values = proof.public_values.raw();
    // println!("public values: {:?}", public_values);

    // // Get the proof as bytes.
    // let solidity_proof = proof.raw();
    // println!("proof: {:?}", solidity_proof);

    // Verify proof and public values
    // client.verify(&proof, &vk).expect("verification failed");

    // Save the proof.
    // proof.save("proof-with-pis.bin").expect("saving proof failed");

    println!("successfully generated and verified proof for the program!")
}

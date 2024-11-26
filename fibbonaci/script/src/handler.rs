use actix_web::{get, post};
use actix_web::{get, http::StatusCode, post, web, App, HttpResponse, HttpServer, Responder};
use ethers::core::types::{Address, U256};
use ethers::types::Bytes;
use ethers::{
    core::k256::ecdsa::SigningKey,
    signers::{LocalWallet, Signer, Wallet},
};
use ethers::abi::{decode, AbiType, Token};
use ethers::types::Bytes;
use reqwest::blocking::Client;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use sp1_sdk::{utils, ProverClient, SP1Proof, SP1Stdin};
use std::fs;
use std::vec;
use uuid::Uuid;

use actix_web::HttpResponse;

/// The ELF we want to execute inside the zkVM.
const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

#[get("/test")]
async fn test() -> impl Responder {
    common::response("Generator is running", StatusCode::OK, None)
}

#[get("/benchmark")]
async fn benchmark() -> impl Responder {
    common::response(
        "Benchmarking API is not implemented",
        StatusCode::NOT_IMPLEMENTED,
        None,
    )
}

async fn process_proof(payload: kalypso_generator_models::models::InputPayload) -> impl Responder {

    utils::setup_logger();
    // Convert Vec<u8> to String
    let json_string = String::from_utf8(payload.get_plain_secrets().unwrap())?;

    // Deserialize JSON string to serde_json::Value
    let json_value: Value = serde_json::from_str(&json_string)?;

    // Extract `n` as a u32
    let n = json_value
    .get("n") // Get the value associated with the key "n"
    .and_then(Value::as_u64) // Convert it to a u64 (JSON doesn't have u32)
    .ok_or("Key 'n' missing or not a valid number")? as u32; // Convert u64 to u32

    // n to be created from json value
    let mut stdin = SP1Stdin::new();
    stdin.write(&n);

    // Generate the proof for the given program and input.
    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);
    let mut proof = client.prove(&pk, stdin).unwrap();

    println!("generated proof");

    // Read and verify the output.
    let _ = proof.public_values.read::<u32>();
    let a = proof.public_values.read::<u32>();
    let b = proof.public_values.read::<u32>();

    println!("a: {}", a);
    println!("b: {}", b);

    // Verify proof and public values
    client.verify(&proof, &vk).expect("verification failed");

    // Generate a unique filename using uuid
    let filename = format!("proof-{}.bin", Uuid::new_v4());

    // Save the proof to the file
    proof.save(&filename).expect("saving proof failed");

    let deserialized_proof = SP1Proof::load(&filename).expect("loading proof failed");
    // Verify the deserialized proof.
    client
        .verify(&deserialized_proof, &vk)
        .expect("verification failed");

    println!("successfully generated and verified proof for the program!");    

    // Read the file as bytes
    let proof_bytes = fs::read(&filename).expect("reading proof file failed");
    let file_bytes = Bytes::from(proof_bytes);

    match get_signed_proof(payload.clone(), file_bytes).await {
        Ok(proof_data) => {
            // Construct JSON response with proof data
            return HttpResponse::Ok().json(
                kalypso_generator_models::models::GenerateProofResponse {
                    proof: proof_data.to_vec(),
                },
            );
        }
        Err(err) => HttpResponse::InternalServerError().json(JsonResponse {
            message: format!("Failed to generate signed proof: {}", err),
            data: Bytes::new(),
        }),
    }


}



async fn get_signed_proof(
    inputs: kalypso_generator_models::models::InputPayload,
    proof: Bytes,
) -> Result<Bytes, Box<dyn std::error::Error>> {
    // Read the secp256k1 private key from file
    let read_secp_private_key = fs::read("./app/secp.sec").expect("/app/secp.sec file not found");
    let secp_private_key = secp256k1::SecretKey::from_slice(&read_secp_private_key)
        .expect("Failed reading secp_private_key get_signed_proof()")
        .display_secret()
        .to_string();
    let signer_wallet = secp_private_key
        .parse::<LocalWallet>()
        .expect("Failed creating signer_wallet get_signed_proof()");

    // Prepare the data for signing
    // let public_inputs = inputs.ask.prover_data.clone();
    let public_inputs: ethers::types::Bytes = inputs.clone().get_public().into();
    let proof_bytes = proof.clone();
    println!("{:?}", &proof_bytes);

    // Encode the data for signing
    let value = vec![
        ethers::abi::Token::Bytes(public_inputs.to_vec()),
        ethers::abi::Token::Bytes(proof_bytes.to_vec()),
    ];
    let encoded = ethers::abi::encode(&value);
    let digest = ethers::utils::keccak256(encoded);

    // Sign the message digest
    let signature = signer_wallet
        .sign_message(ethers::types::H256(digest))
        .await
        .expect("Failed creating signature get_signed_proof()");

    let sig_bytes: Bytes = signature.to_vec().into();
    // Encode the proof response
    let value = vec![
        ethers::abi::Token::Bytes(public_inputs.to_vec()),
        ethers::abi::Token::Bytes(proof_bytes.to_vec()),
        ethers::abi::Token::Bytes(sig_bytes.to_vec()),
    ];
    let encoded = ethers::abi::encode(&value);
    Ok(encoded.into())
}

use tokio::sync::Semaphore;
use lazy_static::lazy_static;

lazy_static! {
    static ref SEMAPHORE: Semaphore = Semaphore::new(2);
}

#[post("/generateProof")]
async fn generate_proof(inputs: web::Json<kalypso_generator_models::models::InputPayload>) -> impl Responder {
    // Acquire a permit from the semaphore.
    let _permit = SEMAPHORE.acquire().await.unwrap();

    let input_data = inputs.0.clone();
    process_proof(input_data).await
}

pub fn routes(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api")
        .service(test)
        .service(benchmark)
        .service(generate_proof);

    conf.service(scope);
}

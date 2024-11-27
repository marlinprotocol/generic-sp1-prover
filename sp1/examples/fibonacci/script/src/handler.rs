
use actix_web::{get, http::StatusCode, post, web, App, HttpResponse, HttpServer, Responder};
use ethers::core::types::{Address, U256};
use ethers::types::Bytes;
use ethers::{
    core::k256::ecdsa::SigningKey,
    signers::{LocalWallet, Signer, Wallet},
};
use ethers::abi::{decode, AbiType, Token};

use reqwest::blocking::Client;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use sp1_sdk::{utils, ProverClient, SP1Proof, SP1Stdin};
use std::fs;
use std::vec;
use uuid::Uuid;

// use tokio::sync::Semaphore;
// use lazy_static::lazy_static;
use kalypso_generator_models::models::{GenerateProofResponse, InputPayload};

// lazy_static! {
//     static ref SEMAPHORE: Semaphore = Semaphore::new(2);
// }

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

#[post("/checkInput")]
async fn check_input_handler(
    payload: web::Json<kalypso_generator_models::models::InputPayload>,
)-> impl Responder { 
    return HttpResponse::Ok().json(kalypso_ivs_models::models::CheckInputResponse { valid: true });
}

// #[post("/generateProof")]
// async fn check_input_handler2(
//     payload: web::Json<kalypso_generator_models::models::InputPayload>,
// )-> impl Responder { 
//     return HttpResponse::Ok().json(kalypso_ivs_models::models::CheckInputResponse { valid: true });
// }

#[post("/verifyInputsAndProof")]
async fn verify_inputs_and_proof(
    payload: web::Json<kalypso_ivs_models::models::VerifyInputsAndProof>,
) -> impl Responder {
    let default_response = kalypso_ivs_models::models::VerifyInputAndProofResponse {
        is_input_and_proof_valid: true,
    };
    return HttpResponse::Ok().json(default_response);
}

async fn process_proof(
    payload: InputPayload,
) -> Result<HttpResponse, actix_web::Error> {
    utils::setup_logger();


    println!("logger is set");
    // Convert secrets from `Vec<u8>` to `String`
    let json_string = String::from_utf8(payload.get_plain_secrets().map_err(|_| {
        actix_web::error::ErrorBadRequest("Invalid secrets payload")  // Handle `FromUtf8Error` here
    })?).map_err(|_| {
        actix_web::error::ErrorBadRequest("Failed to convert secrets to UTF-8")  // Handle `FromUtf8Error` here
    })?;

    println!("json string {:?}", json_string);
    // Parse JSON
    let json_value: Value = serde_json::from_str(&json_string).map_err(|_| {
        actix_web::error::ErrorBadRequest("Invalid JSON format")
    })?;

    println!("json value {:?}", json_value);
    // Extract `n` from JSON
    let n = json_value
        .get("n")
        .and_then(|v| v.as_str()?.parse::<u32>().ok())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing or invalid key 'n'"))?;

    println!("value of n extracted {:?}", n);
    // Simulated proof generation
    let mut stdin = SP1Stdin::new();
    stdin.write(&n);

    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);
    let mut proof = client.prove(&pk, stdin).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Proof generation failed")
    })?;

    client
        .verify(&proof, &vk)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Proof verification failed"))?;

    let filename = format!("proof-{}.bin", Uuid::new_v4());
    proof.save(&filename).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Saving proof failed")
    })?;

    // Load and verify proof again
    let deserialized_proof = SP1Proof::load(&filename).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Loading proof failed")
    })?;
    client
        .verify(&deserialized_proof, &vk)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Deserialized proof verification failed"))?;

    // Read proof file as bytes
    let proof_bytes = fs::read(&filename).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Reading proof file failed")
    })?;
    let file_bytes = Bytes::from(proof_bytes);

    // Generate signed proof
    let proof_data = get_signed_proof(payload, file_bytes)
        .await
        .map_err(|err| actix_web::error::ErrorInternalServerError(format!("Signed proof generation failed: {}", err)))?;

    // Return successful response
    Ok(HttpResponse::Ok().json(GenerateProofResponse {
        proof: proof_data.to_vec(),
    }))
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



#[post("/generateProof")]
async fn generate_proof(
    inputs: web::Json<kalypso_generator_models::models::InputPayload>,
) -> impl Responder {
    
    println!("Received payload: {:?}", inputs);
    
    // return HttpResponse::Ok().json(kalypso_ivs_models::models::CheckInputResponse { valid: true });
    
    match process_proof(inputs.0.clone()).await {
        Ok(response) => response,
        Err(err) => err.error_response(),
    }
}

pub fn routes(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api")
        .service(test)
        .service(benchmark)
        .service(check_input_handler)
        .service(verify_inputs_and_proof)
        .service(generate_proof);

    conf.service(scope);
}

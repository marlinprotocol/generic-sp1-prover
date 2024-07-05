use actix_web::{get, post};
use actix_web::{http::StatusCode, web, Responder};

use ethers::abi::{decode, AbiType, Token};
use ethers::types::Bytes;
use reqwest::blocking::Client;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use sp1_sdk::{utils, ProverClient, SP1Proof, SP1Stdin};
use std::fs;
use std::vec;
use uuid::Uuid;

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

async fn process_proof(input_data: Vec<u8>) -> impl Responder {
    let outer_types = vec![
        <ethers::types::U256 as AbiType>::param_type(), // uint256
        <Bytes as AbiType>::param_type(),               // bytes (nested)
    ];

    let outer_decoded: Vec<Token> =
        decode(&outer_types, &input_data).expect("Decoding outer layer failed");

    let num_bytes: ethers::types::U256 = outer_decoded[0]
        .clone()
        .into_uint()
        .expect("Failed to decode U256");
    let num_bytes_usize = num_bytes.as_usize();

    let nested_data: Vec<u8> = outer_decoded[1]
        .clone()
        .into_bytes()
        .expect("Failed to decode nested bytes");

    // Now, decode the nested bytes array
    let nested_types = vec![<Bytes as AbiType>::param_type(); num_bytes_usize];

    let nested_decoded: Vec<Token> =
        decode(&nested_types, &nested_data).expect("Decoding nested bytes array failed");

    if num_bytes_usize == 0 || num_bytes_usize != nested_decoded.len() {
        return common::response(
            "Invalid number of byte inputs",
            StatusCode::BAD_REQUEST,
            None,
        );
    }

    utils::setup_logger();

    // Create a new stdin with the input for the program.
    let mut stdin = SP1Stdin::new();

    for i in 0..=num_bytes_usize - 1 {
        let input: Vec<u8> = nested_decoded[i]
            .clone()
            .into_bytes()
            .expect("Failed to decode Vec<u8>");
        stdin.write(&input);
    }

    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);
    let proof = client.prove(&pk, stdin).expect("proving failed");

    // Verify proof.
    client.verify(&proof, &vk).expect("verification failed");

    // instead of proof-with-pis, use file name generated by uuid, then save file, again read file as bytes and send it in response using hex::encode

    // Generate a unique filename using uuid
    let filename = format!("proof-{}.bin", Uuid::new_v4());

    // Save the proof to the file
    proof.save(&filename).expect("saving proof failed");

    let deserialized_proof = SP1Proof::load(&filename).expect("loading proof failed");
    // Verify the deserialized proof.
    client
        .verify(&deserialized_proof, &vk)
        .expect("verification failed");

    // Read the file as bytes
    let proof_bytes = fs::read(&filename).expect("reading proof file failed");

    // Upload the file to Pinata IPFS and get the link
    let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VySW5mb3JtYXRpb24iOnsiaWQiOiJhZGExNTk5Ny02YTQxLTQ1NWMtOTMzZS0yZGM4MTg3MjY1NjciLCJlbWFpbCI6ImFrc2hheS4xMTFtZWhlckBnbWFpbC5jb20iLCJlbWFpbF92ZXJpZmllZCI6dHJ1ZSwicGluX3BvbGljeSI6eyJyZWdpb25zIjpbeyJpZCI6IkZSQTEiLCJkZXNpcmVkUmVwbGljYXRpb25Db3VudCI6MX0seyJpZCI6Ik5ZQzEiLCJkZXNpcmVkUmVwbGljYXRpb25Db3VudCI6MX1dLCJ2ZXJzaW9uIjoxfSwibWZhX2VuYWJsZWQiOmZhbHNlLCJzdGF0dXMiOiJBQ1RJVkUifSwiYXV0aGVudGljYXRpb25UeXBlIjoic2NvcGVkS2V5Iiwic2NvcGVkS2V5S2V5IjoiOWRjYzQyY2I0ODdmZTBhMTVlNmEiLCJzY29wZWRLZXlTZWNyZXQiOiIwYTE4MWI3NmIzNmIxNDBlMzE1ZWVmOGI1MjIyZjQ1OTk3ODI2NWY2NGYxMmVkMWM2MjdjMTU3MzE2YTUyNzdhIiwiaWF0IjoxNzIwMDA1OTU0fQ.Y3hNEN6ll1FEfpCWSeXoXLOLahYkR7O9I9qgg4mQiks"; // replace with your JWT token

    let form = reqwest::blocking::multipart::Form::new()
        .file("file", &filename)
        .expect("failed to create form")
        .text("pinataMetadata", r#"{"name": "File name"}"#)
        .text("pinataOptions", r#"{"cidVersion": 0}"#);

    let client = Client::new();
    let res = client
        .post("https://api.pinata.cloud/pinning/pinFileToIPFS")
        .header("Authorization", format!("Bearer {}", jwt))
        .multipart(form)
        .send()
        .expect("uploading file failed");

    let response_json: Value = res.json().expect("failed to parse response");
    let ipfs_hash = response_json["IpfsHash"]
        .as_str()
        .expect("failed to get IpfsHash");

    let url = format!("https://gateway.pinata.cloud/ipfs/{}", ipfs_hash);

    // Encode the bytes to a hex string
    let url_hex = hex::encode(url);

    fs::remove_file(&filename).expect("removing proof file failed");

    common::response(
        "Proof Generated",
        StatusCode::OK,
        Some(Value::String(url_hex)),
    )
}

#[derive(Serialize, Debug, Deserialize)]
struct OnlyInput {
    pub input: String,
}

#[post("/customBenchmark")]
async fn generate_custom_benchmark(_jsonbody: web::Json<OnlyInput>) -> impl Responder {
    let input_data = hex::decode(&_jsonbody.input).expect("Failed decoding inputs");
    process_proof(input_data).await
}

use tokio::sync::Semaphore;
use lazy_static::lazy_static;

lazy_static! {
    static ref SEMAPHORE: Semaphore = Semaphore::new(2);
}

#[post("/generateProof")]
async fn generate_proof(_jsonbody: web::Json<common::GenerateProofInputs>) -> impl Responder {
    // Acquire a permit from the semaphore.
    let _permit = SEMAPHORE.acquire().await.unwrap();

    let input_data = _jsonbody.ask.clone().prover_data.to_vec();
    process_proof(input_data).await
}

pub fn routes(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api")
        .service(test)
        .service(benchmark)
        .service(generate_custom_benchmark)
        .service(generate_proof);

    conf.service(scope);
}

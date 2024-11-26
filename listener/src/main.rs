use dotenv::dotenv;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct Config {
    generator_address: String,
    gas_key: String,
    market_id: String,
    http_rpc_url: String,
    proof_marketplace_address: String,
    generator_registry_address: String,
    start_block: u64,
    chain_id: u64,
    max_parallel_proofs: Option<usize>,
    ivs_url: String,
    prover_url: String,
}

fn load_config(file_path: &str) -> Config {
    let content = fs::read_to_string(file_path).expect("Failed to read config file");
    toml::from_str(&content).expect("Failed to parse config file")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = load_config("config.toml");

    let max_parallel_proofs = config.max_parallel_proofs.unwrap_or(1);

    let listener =
        kalypso_listener::job_creator::JobCreator::simple_listener_for_confidential_prover(
            config.generator_address,
            config.market_id.into(),
            config.http_rpc_url.into(),
            config.gas_key,
            config.proof_marketplace_address.into(),
            config.generator_registry_address.into(),
            config.start_block,
            config.chain_id,
            config.prover_url,
            config.ivs_url,
            false,
            max_parallel_proofs,
        );
        pub fn simple_listener_for_confidential_prover(
            generator_address: String,
            ecies_private_key: String,
            supported_market_dec_string: String,
            http_rpc_url: String,
            gas_key: String,
            proof_market_place: String,
            generator_registry: String,
            start_block: u64,
            chain_id: u64,
            prover_port: String,
            enable_logging_server: bool,
            max_threads: usize,
            skip_input_verification: bool,
        )
    let _ = listener.run().await;

    println!("All tasks completed or shutdown.");

    Ok(())
}

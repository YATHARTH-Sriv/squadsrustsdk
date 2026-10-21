use std::env;

use solana_rpc_client::rpc_client::RpcClient;
use squadsrust_sdk::{pda::program_config_pda, program_config::decode_program_config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let rpc_url = env::var("RPC_URL")?;

    let rpc = RpcClient::new(rpc_url.clone());

    let (program_config_address, _) = program_config_pda();

    let account = rpc.get_account(&program_config_address)?;

    let program_config = decode_program_config(&account.owner, &account.data)?;

    println!("ProgramConfig PDA: {program_config_address}");

    println!("Account owner: {}", account.owner);

    println!(
        "Smart account index: {}",
        program_config.smart_account_index
    );

    println!("Authority: {}", program_config.authority);

    println!(
        "Smart account creation fee: {} lamports",
        program_config.smart_account_creation_fee
    );

    println!("Treasury: {}", program_config.treasury);

    Ok(())
}

use std::{env, str::FromStr};

use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use squadsrust_sdk::settings::decode_settings_account;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let rpc = env::var("RPC_URL")?;

    let rpc_client = RpcClient::new(rpc.clone());

    let setting_addr = env::var("SETTINGS_ADDRESS")?;

    let settings_pubkey = Pubkey::from_str(&setting_addr)?;

    println!("Setting ddress is {settings_pubkey}");

    let account = rpc_client.get_account(&settings_pubkey)?;

    let fetch_settings = decode_settings_account(&account.owner, &account.data)?;

    println!("Settings Fields {:?}", fetch_settings);
    Ok(())
}

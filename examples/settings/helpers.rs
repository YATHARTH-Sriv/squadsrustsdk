use std::{env, str::FromStr};

use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use squadsrust_sdk::settings::{SettingsAccount, decode_settings_account};

pub struct HelperResponse {
    pub rpc: RpcClient,
    pub decoded_settings: SettingsAccount,
    pub settingpubkey: Pubkey,
}

pub fn create_rpc_and_settings_account() -> Result<HelperResponse, Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let rpc_url = env::var("RPC_URL")?;

    let rpc = RpcClient::new(rpc_url.clone());

    let setting_addr = env::var("SETTINGS_ADDRESS")?;

    let settingpubkey = Pubkey::from_str(&setting_addr)?;

    let account = rpc.get_account(&settingpubkey)?;

    let decoded_settings = decode_settings_account(&account.owner, &account.data)?;

    Ok(HelperResponse {
        rpc,
        decoded_settings,
        settingpubkey,
    })
}

use std::env;

use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::{signature::read_keypair_file, signer::Signer};
use squadsrust_sdk::{
    pda::program_config_pda,
    program_config::decode_program_config,
    settings::{
        CreateSmartAccountArgs, build_create_smart_account_transaction, decode_settings_account,
    },
    types::{Permissions, SmartAccountSigner},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let rpc_url = env::var("RPC_URL")?;
    let keypair_path = env::var("KEYPAIR_PATH")?;

    let rpc = RpcClient::new(rpc_url.clone());

    let payer = read_keypair_file(keypair_path)?;

    let creator = payer.pubkey();

    let (program_config_address, _) = program_config_pda();

    let program_config_account = rpc.get_account(&program_config_address)?;

    let program_config =
        decode_program_config(&program_config_account.owner, &program_config_account.data)?;

    let recent_blockhash = rpc.get_latest_blockhash()?;

    let args = CreateSmartAccountArgs {
        settings_authority: Some(creator),
        threshold: 1,
        signers: vec![SmartAccountSigner {
            key: creator,
            permissions: Permissions { mask: 0b0000_0111 },
        }],
        time_lock: 0,
        rent_collector: Some(creator),
        memo: Some("Created through the Rust SDK ".to_owned()),
    };

    let output = build_create_smart_account_transaction(
        creator,
        recent_blockhash,
        &program_config,
        creator,
        args,
    )?;

    let settings_address = output.settings;
    let settings_seed = output.settings_seed;

    println!("New Settings seed: {settings_seed}");
    println!("New Settings address: {settings_address}");

    let mut transaction = output.transaction;

    transaction.try_sign(&[&payer], recent_blockhash)?;

    let _signature = rpc.send_and_confirm_transaction(&transaction)?;

    let settings_account = rpc.get_account(&settings_address)?;

    let settings = decode_settings_account(&settings_account.owner, &settings_account.data)?;

    println!("Decoded Settings: {:?}", settings);

    Ok(())
}

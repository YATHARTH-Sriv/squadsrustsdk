use std::{env, str::FromStr};

use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::read_keypair_file, signer::Signer};
use solana_system_interface::instruction::transfer;
use squadsrust_sdk::{
    pda::smart_account_pda,
    settings::decode_settings_account,
    transaction::{CreateTransactionInput, build_create_transaction, decode_transaction_account},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let rpc_url = env::var("RPC_URL")?;
    let rpc = RpcClient::new(rpc_url.clone());

    let keypar_path = env::var("KEYPAIR_PATH")?;
    let payer = read_keypair_file(keypar_path)?;
    let settings_address = Pubkey::from_str(&env::var("SETTINGS_ADDRESS")?)?;

    let settings_rpc_account = rpc.get_account(&settings_address)?;

    let settings =
        decode_settings_account(&settings_rpc_account.owner, &settings_rpc_account.data)?;

    let creator_signer = settings
        .signers
        .iter()
        .find(|signer| signer.key == payer.pubkey())
        .ok_or("the loaded keypair is not a signer in this Settings account")?;

    const INITIATE_PERMISSION: u8 = 1 << 0;

    if creator_signer.permissions.mask & INITIATE_PERMISSION == 0 {
        return Err("the loaded signer does not have Initiate permission".into());
    }

    let recipient = Pubkey::from_str("9KqA4ptqyFHn4jXMbRDCKbojVvjnzs9kSV7zgiEdrKXe")?;

    let transfer_lamports: u64 = 1000000;

    let account_index = 0_u8;

    let (smart_account, _) = smart_account_pda(&settings_address, account_index);

    let transfer_instruction = transfer(&smart_account, &recipient, transfer_lamports);

    println!("Settings: {settings_address}");
    println!("Smart Account: {smart_account}");
    println!("Recipient: {recipient}");
    println!("Transfer amount: {transfer_lamports} lamports");

    let recent_blockhash = rpc.get_latest_blockhash()?;

    let output = build_create_transaction(CreateTransactionInput {
        fee_payer: payer.pubkey(),
        recent_blockhash,
        settings_address,
        settings: &settings,
        creator: payer.pubkey(),
        rent_payer: payer.pubkey(),
        account_index,
        inner_instructions: &[transfer_instruction],
        memo: Some("SOL transfer created with the Rust SDK".to_owned()),
    })?;

    println!("Transaction index: {}", output.transaction_index);
    println!("Transaction address: {}", output.transaction_address);
    println!("Smart Account address: {}", output.smart_account);

    let transaction_address = output.transaction_address;
    let expected_transaction_index = output.transaction_index;

    let mut transaction = output.unsigned_transaction;

    transaction.try_sign(&[&payer], recent_blockhash)?;

    let signature = rpc.send_and_confirm_transaction(&transaction)?;

    println!("Creation signature: {signature}");

    let transaction_rpc_account = rpc.get_account(&transaction_address)?;

    let decoded_transaction = decode_transaction_account(
        &transaction_rpc_account.owner,
        &transaction_rpc_account.data,
    )?;

    println!("Decoded transaction: {decoded_transaction:#?}");

    if decoded_transaction.index != expected_transaction_index {
        return Err("decoded transaction index does not match builder output".into());
    }

    if decoded_transaction.consensus_account != settings_address {
        return Err("decoded transaction belongs to a different Settings account".into());
    }

    Ok(())
}

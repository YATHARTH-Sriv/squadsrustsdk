use std::{env, str::FromStr};

use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::read_keypair_file, signer::Signer};
use squadsrust_sdk::{
    proposal::{CreateProposalInput, build_create_proposal, decode_proposal_account},
    settings::decode_settings_account,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let rpc_url = env::var("RPC_URL")?;
    let rpc = RpcClient::new(rpc_url.clone());

    let keypair_path = env::var("KEYPAIR_PATH")?;
    let payer = read_keypair_file(keypair_path)?;

    let setting_address = Pubkey::from_str(&env::var("SETTINGS_ADDRESS")?)?;
    let txn_index: u64 = env::var("TRANSACTION_INDEX")?.parse()?;

    let settings_account = rpc.get_account(&setting_address)?;
    let settings = decode_settings_account(&settings_account.owner, &settings_account.data)?;

    const INITIATE_PERMISSION: u8 = 1 << 0;
    const VOTE_PERMISSION: u8 = 1 << 1;

    let creator_entry = settings
        .signers
        .iter()
        .find(|signer| signer.key == payer.pubkey())
        .ok_or("the loaded keypair is not a signer in this Settings account")?;

    let allowed_permissions = INITIATE_PERMISSION | VOTE_PERMISSION;

    if creator_entry.permissions.mask & allowed_permissions == 0 {
        return Err("the signer needs Initiate or Vote permission".into());
    }

    // The transaction must already exist.
    if txn_index == 0 || txn_index > settings.transaction_index {
        return Err(format!(
            "transaction index {txn_index} does not exist; latest index is {}",
            settings.transaction_index
        )
        .into());
    }

    // Old transactions can become stale after settings changes.
    if txn_index <= settings.stale_transaction_index {
        return Err(format!(
            "transaction index {txn_index} is stale; stale index is {}",
            settings.stale_transaction_index
        )
        .into());
    }

    let recent_blockhash = rpc.get_latest_blockhash()?;

    let output = build_create_proposal(CreateProposalInput {
        fee_payer: payer.pubkey(),
        recent_blockhash,
        consensus_account: setting_address,
        creator: creator_entry.key,
        rent_payer: payer.pubkey(),
        transaction_index: txn_index,

        // Keep it false because the SDK does not implement activate_proposal currently .
        draft: false,
    })?;
    println!("Settings address: {setting_address}");
    println!("Transaction index: {}", output.transaction_index);
    println!("Proposal address: {}", output.proposal_address);

    let proposal_address = output.proposal_address;
    let mut transaction = output.unsigned_transaction;

    transaction.try_sign(&[&payer], recent_blockhash)?;

    let _signature = rpc.send_and_confirm_transaction(&transaction)?;

    let proposal_account = rpc.get_account(&proposal_address)?;

    let proposal = decode_proposal_account(&proposal_account.owner, &proposal_account.data)?;

    println!("Decoded proposal {:?}", proposal);

    Ok(())
}

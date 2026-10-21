use std::{env, str::FromStr};

use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use squadsrust_sdk::{pda::proposal_pda, proposal::decode_proposal_account};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let rpc_url = env::var("RPC_URL")?;
    let rpc = RpcClient::new(rpc_url.clone());

    let setting_address = Pubkey::from_str(&env::var("SETTINGS_ADDRESS")?)?;
    let txn_index: u64 = env::var("TRANSACTION_INDEX")?.parse()?;

    let (proposal_pda, _bump) = proposal_pda(&setting_address, txn_index);
    println!("Settings address: {setting_address}");
    println!("Transaction index: {txn_index}");
    println!("Proposal address: {proposal_pda}");

    let proposal_account = rpc.get_account(&proposal_pda)?;
    let proposal = decode_proposal_account(&proposal_account.owner, &proposal_account.data)?;

    if proposal.settings != setting_address {
        return Err("proposal belongs to a different Settings account".into());
    }

    if proposal.transaction_index != txn_index {
        return Err("proposal has an unexpected transaction index".into());
    }

    println!("Status: {:?}", proposal.status);
    println!("Approved by: {:?}", proposal.approved);
    println!("Rejected by: {:?}", proposal.rejected);
    println!("Cancelled by: {:?}", proposal.cancelled);

    Ok(())
}

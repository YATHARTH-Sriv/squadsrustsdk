use std::{env, str::FromStr};

use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::read_keypair_file, signer::Signer};
use squadsrust_sdk::{
    pda::proposal_pda,
    proposal::{
        ProposalStatus, VoteOnProposalInput, build_reject_proposal, decode_proposal_account,
    },
    settings::decode_settings_account,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let rpc_url = env::var("RPC_URL")?;
    let rpc = RpcClient::new(rpc_url.clone());

    let settings_addr = Pubkey::from_str(&env::var("SETTINGS_ADDRESS")?)?;
    let txn_index: u64 = env::var("TRANSACTION_INDEX")?.parse()?;

    let payer = read_keypair_file(env::var("KEYPAIR_PATH")?)?;

    let settings_account = rpc.get_account(&settings_addr)?;
    let settings = decode_settings_account(&settings_account.owner, &settings_account.data)?;

    const VOTE_PERMISSION: u8 = 1 << 1;

    let voter_entry = settings
        .signers
        .iter()
        .find(|signer| signer.key == payer.pubkey())
        .ok_or("the loaded keypair is not a signer in this Settings account")?;

    if voter_entry.permissions.mask & VOTE_PERMISSION == 0 {
        return Err("the signer does not have Vote permission".into());
    }

    if txn_index <= settings.stale_transaction_index {
        return Err("cannot vote on a stale proposal".into());
    }

    let voter_count = settings
        .signers
        .iter()
        .filter(|candidate| candidate.permissions.mask & VOTE_PERMISSION != 0)
        .count();

    let rejection_cutoff = voter_count
        .checked_sub(usize::from(settings.threshold))
        .and_then(|value| value.checked_add(1))
        .ok_or("invalid threshold or voter count")?;

    let (proposal_address, _) = proposal_pda(&settings_addr, txn_index);

    let before_rpc_account = rpc.get_account(&proposal_address)?;

    let before = decode_proposal_account(&before_rpc_account.owner, &before_rpc_account.data)?;

    println!("Proposal before rejection: {:?}", before.rejected);
    println!("Rejection cutoff: {rejection_cutoff}");

    if !matches!(&before.status, ProposalStatus::Active { .. }) {
        return Err(format!(
            "only an Active proposal can be rejected; current status is {:?}",
            before.status
        )
        .into());
    }

    if before.rejected.contains(&voter_entry.key) {
        return Err("this signer has already rejected the proposal".into());
    }

    let recent_blockhash = rpc.get_latest_blockhash()?;

    let output = build_reject_proposal(VoteOnProposalInput {
        fee_payer: payer.pubkey(),
        recent_blockhash,
        consensus_account: settings_addr,
        signer: voter_entry.key,
        transaction_index: txn_index,
        memo: Some("Rejected through the Rust SDK".to_owned()),
    })?;

    let mut transaction = output.unsigned_transaction;

    transaction.try_sign(&[&payer], recent_blockhash)?;

    let _signature = rpc.send_and_confirm_transaction(&transaction)?;

    let after_rpc_account = rpc.get_account(&proposal_address)?;

    let after = decode_proposal_account(&after_rpc_account.owner, &after_rpc_account.data)?;

    println!("Proposal After Rejection {:?}", after.rejected);

    Ok(())
}

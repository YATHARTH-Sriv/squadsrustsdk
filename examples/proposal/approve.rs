use std::{env, str::FromStr};

use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::read_keypair_file, signer::Signer};
use squadsrust_sdk::{
    pda::proposal_pda,
    proposal::{
        ProposalStatus, VoteOnProposalInput, build_approve_proposal, decode_proposal_account,
    },
    settings::decode_settings_account,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let rpc_url = env::var("RPC_URL")?;
    let rpc = RpcClient::new(rpc_url.clone());

    let settng_addr = Pubkey::from_str(&env::var("SETTINGS_ADDRESS")?)?;
    let txn_index: u64 = env::var("TRANSACTION_INDEX")?.parse()?;

    let payer = read_keypair_file(env::var("KEYPAIR_PATH")?)?;

    let settings_rpc_account = rpc.get_account(&settng_addr)?;
    let settings =
        decode_settings_account(&settings_rpc_account.owner, &settings_rpc_account.data)?;

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

    let (proposal_addr, _) = proposal_pda(&settng_addr, txn_index);

    let before_proposal_account = rpc.get_account(&proposal_addr)?;

    let before_proposal = decode_proposal_account(
        &before_proposal_account.owner,
        &before_proposal_account.data,
    )?;

    println!("Proposal Before Voted {:?}", before_proposal.approved);

    if !matches!(&before_proposal.status, ProposalStatus::Active { .. }) {
        return Err(format!(
            "only an Active proposal can be approved; current status is {:?}",
            before_proposal.status
        )
        .into());
    }

    if before_proposal.approved.contains(&voter_entry.key) {
        return Err("this signer has already approved the proposal".into());
    }

    let recent_hash = rpc.get_latest_blockhash()?;

    let result = build_approve_proposal(VoteOnProposalInput {
        fee_payer: payer.pubkey(),
        recent_blockhash: recent_hash,
        consensus_account: settng_addr,
        signer: voter_entry.key,
        transaction_index: txn_index,
        memo: Some("Approve Vote from Rust SDK".to_owned()),
    })?;

    let mut transaction = result.unsigned_transaction;

    transaction.try_sign(&[&payer], recent_hash)?;
    let _signature = rpc.send_and_confirm_transaction(&transaction)?;

    let after_proposal_account = rpc.get_account(&proposal_addr)?;

    let after_proposal =
        decode_proposal_account(&after_proposal_account.owner, &after_proposal_account.data)?;

    if !after_proposal.approved.contains(&voter_entry.key) {
        return Err("the approval was not recorded in the Proposal account".into());
    }

    println!("Proposal after approval: {:?} ", after_proposal.approved);

    Ok(())
}

use std::{env, str::FromStr};

use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::read_keypair_file, signer::Signer};

use squadsrust_sdk::{
    pda::{proposal_pda, smart_account_pda, transaction_pda},
    proposal::{ProposalStatus, decode_proposal_account},
    settings::decode_settings_account,
    transaction::{ExecuteTransactionInput, build_execute_transaction, decode_transaction_account},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let rpc_url = env::var("RPC_URL")?;
    let keypair_path = env::var("KEYPAIR_PATH")?;

    let settings_address = Pubkey::from_str(&env::var("SETTINGS_ADDRESS")?)?;

    let transaction_index = env::var("TRANSACTION_INDEX")?.parse::<u64>()?;

    let recipient = Pubkey::from_str("9KqA4ptqyFHn4jXMbRDCKbojVvjnzs9kSV7zgiEdrKXe")?;

    let transfer_lamports = "1000000".parse::<u64>()?;

    let rpc = RpcClient::new(rpc_url);
    let payer = read_keypair_file(keypair_path)?;
    let executor = payer.pubkey();

    if executor == recipient {
        return Err("use a recipient different from the executor for this example".into());
    }

    /*
     * Load Settings and verify that the selected signer is allowed
     * to execute transactions.
     */
    let settings_rpc_account = rpc.get_account(&settings_address)?;

    let settings =
        decode_settings_account(&settings_rpc_account.owner, &settings_rpc_account.data)?;

    const EXECUTE_PERMISSION: u8 = 1 << 2;

    let executor_entry = settings
        .signers
        .iter()
        .find(|signer| signer.key == executor)
        .ok_or("the loaded keypair is not a Settings signer")?;

    if executor_entry.permissions.mask & EXECUTE_PERMISSION == 0 {
        return Err("the signer does not have Execute permission".into());
    }

    /*
     * Load and decode the stored Transaction account.
     */
    let (transaction_address, _) = transaction_pda(&settings_address, transaction_index);

    let transaction_rpc_account = rpc.get_account(&transaction_address)?;

    let transaction_account = decode_transaction_account(
        &transaction_rpc_account.owner,
        &transaction_rpc_account.data,
    )?;

    if transaction_account.consensus_account != settings_address {
        return Err("transaction belongs to another Settings account".into());
    }

    if transaction_account.index != transaction_index {
        return Err("transaction index does not match".into());
    }

    /*
     * Load the associated Proposal. Execution requires Approved status.
     */
    let (proposal_address, _) = proposal_pda(&settings_address, transaction_index);

    let proposal_rpc_account = rpc.get_account(&proposal_address)?;

    let proposal =
        decode_proposal_account(&proposal_rpc_account.owner, &proposal_rpc_account.data)?;

    let approved_timestamp = match &proposal.status {
        ProposalStatus::Approved { timestamp } => *timestamp,

        status => {
            return Err(format!(
                "proposal must be Approved before execution; current status is {status:?}"
            )
            .into());
        }
    };

    /*
     * Check the time lock using the RPC node's block time.
     */
    let current_slot = rpc.get_slot()?;
    let current_timestamp = rpc.get_block_time(current_slot)?;

    let execution_timestamp = approved_timestamp
        .checked_add(i64::from(settings.time_lock))
        .ok_or("time lock calculation overflowed")?;

    if current_timestamp < execution_timestamp {
        let seconds_remaining = execution_timestamp - current_timestamp;

        return Err(format!(
            "time lock has not been released; wait approximately {seconds_remaining} seconds"
        )
        .into());
    }

    /*
     * Derive the Smart Account that will sign the stored inner transfer.
     */
    let (smart_account, _) =
        smart_account_pda(&settings_address, transaction_account.payload.account_index);

    let smart_account_balance_before = rpc.get_balance(&smart_account)?;

    let recipient_balance_before = rpc.get_balance(&recipient)?;

    println!("Transaction: {transaction_address}");
    println!("Proposal: {proposal_address}");
    println!("Smart Account: {smart_account}");
    println!("Executor: {executor}");
    println!("Smart Account balance before: {smart_account_balance_before}");
    println!("Recipient balance before: {recipient_balance_before}");

    if smart_account_balance_before < transfer_lamports {
        return Err(format!(
            "Smart Account has {smart_account_balance_before} lamports but the stored transfer requires {transfer_lamports}"
        )
        .into());
    }

    /*
     * Build the outer unsigned execution transaction.
     *
     * The SDK reconstructs the dynamic remaining accounts from the
     * message stored inside TransactionAccount.
     */
    let recent_blockhash = rpc.get_latest_blockhash()?;

    let output = build_execute_transaction(ExecuteTransactionInput {
        fee_payer: payer.pubkey(),
        recent_blockhash,
        signer: executor,
        transaction: &transaction_account,
    })?;

    if output.transaction_address != transaction_address {
        return Err("builder derived an unexpected transaction address".into());
    }

    if output.proposal_address != proposal_address {
        return Err("builder derived an unexpected proposal address".into());
    }

    let mut transaction = output.unsigned_transaction;

    transaction.try_sign(&[&payer], recent_blockhash)?;

    let simulation = rpc.simulate_transaction(&transaction)?;

    if let Some(error) = simulation.value.err {
        return Err(format!("transaction execution simulation failed: {error:?}").into());
    }

    let signature = rpc.send_and_confirm_transaction(&transaction)?;

    println!("Execution signature: {signature}");

    /*
     * Verify the effects.
     */
    let smart_account_balance_after = rpc.get_balance(&smart_account)?;

    let recipient_balance_after = rpc.get_balance(&recipient)?;

    let updated_proposal_rpc_account = rpc.get_account(&proposal_address)?;

    let updated_proposal = decode_proposal_account(
        &updated_proposal_rpc_account.owner,
        &updated_proposal_rpc_account.data,
    )?;

    if !matches!(&updated_proposal.status, ProposalStatus::Executed { .. }) {
        return Err(format!(
            "expected Executed status, received {:?}",
            updated_proposal.status
        )
        .into());
    }

    let spent = smart_account_balance_before
        .checked_sub(smart_account_balance_after)
        .ok_or("Smart Account balance unexpectedly increased")?;

    let received = recipient_balance_after
        .checked_sub(recipient_balance_before)
        .ok_or("recipient balance unexpectedly decreased")?;

    if spent != transfer_lamports {
        return Err(format!(
            "expected Smart Account to spend {transfer_lamports}, but it spent {spent}"
        )
        .into());
    }

    if received != transfer_lamports {
        return Err(format!(
            "expected recipient to receive {transfer_lamports}, but it received {received}"
        )
        .into());
    }

    println!("Smart Account balance after: {smart_account_balance_after}");
    println!("Recipient balance after: {recipient_balance_after}");
    println!("Proposal status: {:?}", updated_proposal.status);

    Ok(())
}

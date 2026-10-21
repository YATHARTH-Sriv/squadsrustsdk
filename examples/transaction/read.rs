use std::{env, str::FromStr};

use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use squadsrust_sdk::{
    pda::{smart_account_pda, transaction_pda},
    transaction::decode_transaction_account,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let rpc_url = env::var("RPC_URL")?;
    let settings_address = Pubkey::from_str(&env::var("SETTINGS_ADDRESS")?)?;
    let transaction_index = env::var("TRANSACTION_INDEX")?.parse::<u64>()?;

    let rpc = RpcClient::new(rpc_url);

    let (transaction_address, transaction_bump) =
        transaction_pda(&settings_address, transaction_index);

    let rpc_account = rpc.get_account(&transaction_address)?;

    let transaction = decode_transaction_account(&rpc_account.owner, &rpc_account.data)?;

    if transaction.consensus_account != settings_address {
        return Err("transaction belongs to a different Settings account".into());
    }

    if transaction.index != transaction_index {
        return Err("transaction index does not match".into());
    }

    let (smart_account, smart_account_bump) =
        smart_account_pda(&settings_address, transaction.payload.account_index);

    println!("Transaction address: {transaction_address}");
    println!("Transaction PDA bump: {transaction_bump}");
    println!("Settings address: {}", transaction.consensus_account);
    println!("Transaction index: {}", transaction.index);
    println!("Creator: {}", transaction.creator);
    println!("Rent collector: {}", transaction.rent_collector);
    println!("Account index: {}", transaction.payload.account_index);
    println!("Smart Account: {smart_account}");
    println!("Smart Account bump: {smart_account_bump}");

    let message = &transaction.payload.message;

    println!("Required message signers: {}", message.num_signers);
    println!("Writable message signers: {}", message.num_writable_signers);
    println!("Writable non-signers: {}", message.num_writable_non_signers);

    println!("Account keys:");

    for (index, key) in message.account_keys.iter().enumerate() {
        println!("  [{index}] {key}");
    }

    println!("Compiled instructions:");

    for (index, instruction) in message.instructions.iter().enumerate() {
        println!("  Instruction {index}:");
        println!("    Program index: {}", instruction.program_id_index);
        println!("    Account indexes: {:?}", instruction.account_indexes);
        println!("    Data: {:?}", instruction.data);
    }

    println!(
        "Address table lookups: {}",
        message.address_table_lookups.len()
    );

    println!(
        "Ephemeral signers: {}",
        transaction.payload.ephemeral_signer_bumps.len()
    );

    Ok(())
}

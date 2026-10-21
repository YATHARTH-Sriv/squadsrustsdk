use solana_sdk::{pubkey::Pubkey, signature::read_keypair_file, signer::Signer};
use squadsrust_sdk::settings::{
    AuthoritySettingsAccounts, RemoveSignerArgs, build_remove_signer_as_authority_transaction,
    decode_settings_account,
};
use std::{env, str::FromStr};

mod helpers;
use helpers::{HelperResponse, create_rpc_and_settings_account};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let resp: HelperResponse = create_rpc_and_settings_account()?;
    let rpc = resp.rpc;
    let before_settings_account = resp.decoded_settings;

    println!(
        "Before removing signer {:?}",
        before_settings_account.signers
    );

    let signer_to_be_removed = Pubkey::from_str("J2yXfRiosWrS7UKiXYAgTgvNF7wEgjwM7tbAJaHopGQn")?;

    let recent_hash = rpc.get_latest_blockhash()?;

    let keypair_path = env::var("KEYPAIR_PATH")?;
    let payer = read_keypair_file(keypair_path)?;

    let mut txn = build_remove_signer_as_authority_transaction(
        payer.pubkey(),
        recent_hash,
        AuthoritySettingsAccounts {
            settings: resp.settingpubkey,
            settings_authority: before_settings_account.settings_authority,
            rent_payer: Some(payer.pubkey()),
        },
        RemoveSignerArgs {
            old_signer: signer_to_be_removed,
            memo: Some("Add signer Rust example".to_owned()),
        },
    )?;
    txn.try_sign(&[payer], recent_hash)?;

    let _sign = rpc.send_and_confirm_transaction(&txn)?;

    let updated_account = rpc.get_account(&resp.settingpubkey)?;

    let after_settings_account =
        decode_settings_account(&updated_account.owner, &updated_account.data)?;

    println!(
        "The Settings Account After Removing Signer {:?}",
        after_settings_account.signers
    );

    Ok(())
}

use std::env;

use solana_sdk::{signature::read_keypair_file, signer::Signer};
use squadsrust_sdk::settings::{
    AuthoritySettingsAccounts, ChangeThresholdArgs,
    build_change_threshold_as_authority_transaction, decode_settings_account,
};

mod helpers;
use helpers::{HelperResponse, create_rpc_and_settings_account};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let resp: HelperResponse = create_rpc_and_settings_account()?;
    let rpc = resp.rpc;
    let before_settings_account = resp.decoded_settings;

    println!(
        "The Settings Account BEFORE {:?}",
        before_settings_account.threshold
    );

    let new: u16 = 2;
    let no_of_signers = before_settings_account.signers.len() as u16;

    if new <= no_of_signers {
        let recent_hash = rpc.get_latest_blockhash()?;

        let keypair_path = env::var("KEYPAIR_PATH")?;
        let payer = read_keypair_file(keypair_path)?;

        let mut txn = build_change_threshold_as_authority_transaction(
            payer.pubkey(),
            recent_hash,
            AuthoritySettingsAccounts {
                settings: resp.settingpubkey,
                settings_authority: before_settings_account.settings_authority,
                rent_payer: Some(before_settings_account.settings_authority),
            },
            ChangeThresholdArgs {
                new_threshold: new,
                memo: Some("New threshold from saud sdk".to_owned()),
            },
        )?;

        txn.try_sign(&[payer], txn.message.recent_blockhash)?;

        let _signature = rpc.send_and_confirm_transaction(&txn)?;

        let updated_account = rpc.get_account(&resp.settingpubkey)?;

        let after_settings_account =
            decode_settings_account(&updated_account.owner, &updated_account.data)?;

        println!(
            "The Settings Account After Added Signer {:?}",
            after_settings_account.threshold
        );
    }

    Ok(())
}

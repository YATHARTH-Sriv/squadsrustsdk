use solana_sdk::{pubkey::Pubkey, signature::read_keypair_file, signer::Signer};
use squadsrust_sdk::{
    settings::{
        AddSignerArgs, AuthoritySettingsAccounts, build_add_signer_as_authority_transaction,
        decode_settings_account,
    },
    types::{Permissions, SmartAccountSigner},
};
use std::{env, str::FromStr};

mod helpers;
use helpers::{HelperResponse, create_rpc_and_settings_account};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let resp: HelperResponse = create_rpc_and_settings_account()?;
    let rpc = resp.rpc;
    let before_settings_account = resp.decoded_settings;

    println!("The Settings Account BEFORE {:?}", before_settings_account);

    let signer_to_be_added = Pubkey::from_str("J2yXfRiosWrS7UKiXYAgTgvNF7wEgjwM7tbAJaHopGQn")?;

    let recent_hash = rpc.get_latest_blockhash()?;

    let keypair_path = env::var("KEYPAIR_PATH")?;
    let payer = read_keypair_file(keypair_path)?;

    let mut txn = build_add_signer_as_authority_transaction(
        payer.pubkey(),
        recent_hash,
        AuthoritySettingsAccounts {
            settings: resp.settingpubkey,
            settings_authority: before_settings_account.settings_authority,
            rent_payer: Some(before_settings_account.settings_authority),
        },
        AddSignerArgs {
            new_signer: SmartAccountSigner {
                key: signer_to_be_added,
                permissions: Permissions { mask: 0b0000_0111 },
            },
            memo: Some("Add signer Rust example".to_owned()),
        },
    )?;

    txn.try_sign(&[payer], txn.message.recent_blockhash)?;

    let _signature = rpc.send_and_confirm_transaction(&txn)?;

    let updated_account = rpc.get_account(&resp.settingpubkey)?;

    let after_settings_account =
        decode_settings_account(&updated_account.owner, &updated_account.data)?;

    println!(
        "The Settings Account After Added Signer {:?}",
        after_settings_account
    );

    Ok(())
}

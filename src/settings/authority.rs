use borsh::BorshSerialize;
use solana_sdk::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    transaction::Transaction,
};

use crate::{
    constant::{PROGRAM_ID, SYSTEM_PROGRAM_ID},
    error::SdkError,
    helpers::anchor_discriminator,
    transaction::build_unsigned_transaction,
    types::SmartAccountSigner,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthoritySettingsAccounts {
    pub settings: Pubkey,
    pub settings_authority: Pubkey,
    pub rent_payer: Option<Pubkey>,
}

fn build_authority_settings_instruction<T: BorshSerialize>(
    instruction_name: &str,
    accounts: AuthoritySettingsAccounts,
    args: &T,
) -> Result<Instruction, SdkError> {
    let discriminator = anchor_discriminator(instruction_name);

    let encoded_args = borsh::to_vec(args).map_err(SdkError::AuthoritySettingsSerialization)?;

    let mut data = Vec::with_capacity(discriminator.len() + encoded_args.len());

    data.extend_from_slice(&discriminator);
    data.extend_from_slice(&encoded_args);

    let (rent_payer, system_program) = match accounts.rent_payer {
        Some(rent_payer) => (
            AccountMeta::new(rent_payer, true),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ),
        None => (
            AccountMeta::new_readonly(PROGRAM_ID, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
        ),
    };

    Ok(Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(accounts.settings, false),
            AccountMeta::new_readonly(accounts.settings_authority, true),
            rent_payer,
            system_program,
            AccountMeta::new_readonly(PROGRAM_ID, false),
        ],
        data,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize)]
pub struct ChangeThresholdArgs {
    pub new_threshold: u16,
    pub memo: Option<String>,
}

pub fn build_change_threshold_as_authority_instruction(
    accounts: AuthoritySettingsAccounts,
    args: ChangeThresholdArgs,
) -> Result<Instruction, SdkError> {
    build_authority_settings_instruction("global:change_threshold_as_authority", accounts, &args)
}

pub fn build_change_threshold_as_authority_transaction(
    fee_payer: Pubkey,
    recent_blockhash: Hash,
    accounts: AuthoritySettingsAccounts,
    args: ChangeThresholdArgs,
) -> Result<Transaction, SdkError> {
    let instruction = build_change_threshold_as_authority_instruction(accounts, args)?;

    Ok(build_unsigned_transaction(
        fee_payer,
        recent_blockhash,
        instruction,
    ))
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize)]
pub struct AddSignerArgs {
    pub new_signer: SmartAccountSigner,
    pub memo: Option<String>,
}

pub fn build_add_signer_as_authority_instruction(
    accounts: AuthoritySettingsAccounts,
    args: AddSignerArgs,
) -> Result<Instruction, SdkError> {
    build_authority_settings_instruction("global:add_signer_as_authority", accounts, &args)
}

pub fn build_add_signer_as_authority_transaction(
    fee_payer: Pubkey,
    recent_blockhash: Hash,
    accounts: AuthoritySettingsAccounts,
    args: AddSignerArgs,
) -> Result<Transaction, SdkError> {
    let instruction = build_add_signer_as_authority_instruction(accounts, args)?;

    Ok(build_unsigned_transaction(
        fee_payer,
        recent_blockhash,
        instruction,
    ))
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize)]
pub struct RemoveSignerArgs {
    pub old_signer: Pubkey,
    pub memo: Option<String>,
}

pub fn build_remove_signer_as_authority_instruction(
    accounts: AuthoritySettingsAccounts,
    args: RemoveSignerArgs,
) -> Result<Instruction, SdkError> {
    build_authority_settings_instruction("global:remove_signer_as_authority", accounts, &args)
}

pub fn build_remove_signer_as_authority_transaction(
    fee_payer: Pubkey,
    recent_blockhash: Hash,
    accounts: AuthoritySettingsAccounts,
    args: RemoveSignerArgs,
) -> Result<Transaction, SdkError> {
    let instruction = build_remove_signer_as_authority_instruction(accounts, args)?;

    Ok(build_unsigned_transaction(
        fee_payer,
        recent_blockhash,
        instruction,
    ))
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize)]
pub struct SetTimeLockArgs {
    pub time_lock: u32,
    pub memo: Option<String>,
}

pub fn build_set_time_lock_as_authority_instruction(
    accounts: AuthoritySettingsAccounts,
    args: SetTimeLockArgs,
) -> Result<Instruction, SdkError> {
    build_authority_settings_instruction("global:set_time_lock_as_authority", accounts, &args)
}

pub fn build_set_time_lock_as_authority_transaction(
    fee_payer: Pubkey,
    recent_blockhash: Hash,
    accounts: AuthoritySettingsAccounts,
    args: SetTimeLockArgs,
) -> Result<Transaction, SdkError> {
    let instruction = build_set_time_lock_as_authority_instruction(accounts, args)?;

    Ok(build_unsigned_transaction(
        fee_payer,
        recent_blockhash,
        instruction,
    ))
}

#[cfg(test)]
mod tests {
    use solana_sdk::signature::Signature;

    use super::*;
    use crate::types::Permissions;

    fn authority_settings_accounts(rent_payer: Option<Pubkey>) -> AuthoritySettingsAccounts {
        AuthoritySettingsAccounts {
            settings: Pubkey::new_from_array([20; 32]),
            settings_authority: Pubkey::new_from_array([21; 32]),
            rent_payer,
        }
    }

    fn change_threshold_args() -> ChangeThresholdArgs {
        ChangeThresholdArgs {
            new_threshold: 2,
            memo: Some("hello".to_owned()),
        }
    }

    #[test]
    fn builds_change_threshold_accounts_with_rent_payer() {
        let rent_payer = Pubkey::new_from_array([22; 32]);

        let instruction = build_change_threshold_as_authority_instruction(
            authority_settings_accounts(Some(rent_payer)),
            change_threshold_args(),
        )
        .unwrap();

        assert_eq!(instruction.program_id, PROGRAM_ID);

        assert_eq!(
            instruction.accounts,
            vec![
                AccountMeta::new(Pubkey::new_from_array([20; 32]), false),
                AccountMeta::new_readonly(Pubkey::new_from_array([21; 32]), true,),
                AccountMeta::new(rent_payer, true),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                AccountMeta::new_readonly(PROGRAM_ID, false),
            ]
        );
    }

    #[test]
    fn builds_change_threshold_accounts_without_rent_payer() {
        let instruction = build_change_threshold_as_authority_instruction(
            authority_settings_accounts(None),
            change_threshold_args(),
        )
        .unwrap();

        assert_eq!(
            instruction.accounts,
            vec![
                AccountMeta::new(Pubkey::new_from_array([20; 32]), false),
                AccountMeta::new_readonly(Pubkey::new_from_array([21; 32]), true,),
                AccountMeta::new_readonly(PROGRAM_ID, false),
                AccountMeta::new_readonly(PROGRAM_ID, false),
                AccountMeta::new_readonly(PROGRAM_ID, false),
            ]
        );
    }

    #[test]
    fn serializes_change_threshold_data() {
        let instruction = build_change_threshold_as_authority_instruction(
            authority_settings_accounts(None),
            change_threshold_args(),
        )
        .unwrap();

        let mut expected_data = vec![51, 141, 78, 133, 70, 47, 95, 124];

        // new_threshold
        expected_data.extend_from_slice(&2_u16.to_le_bytes());

        // memo: Some("hello")
        expected_data.push(1);
        expected_data.extend_from_slice(&5_u32.to_le_bytes());
        expected_data.extend_from_slice(b"hello");

        assert_eq!(instruction.data, expected_data);
    }

    #[test]
    fn builds_threshold_transaction_with_one_required_signer() {
        let authority = Pubkey::new_from_array([21; 32]);
        let recent_blockhash = Hash::new_from_array([30; 32]);

        let transaction = build_change_threshold_as_authority_transaction(
            authority,
            recent_blockhash,
            AuthoritySettingsAccounts {
                settings: Pubkey::new_from_array([20; 32]),
                settings_authority: authority,
                rent_payer: Some(authority),
            },
            change_threshold_args(),
        )
        .unwrap();

        assert_eq!(transaction.message.recent_blockhash, recent_blockhash);
        assert_eq!(transaction.message.account_keys[0], authority);

        // Fee payer, Settings Authority and rent payer are the same key.
        assert_eq!(transaction.message.header.num_required_signatures, 1);
        assert_eq!(transaction.signatures.len(), 1);
        assert_eq!(transaction.signatures[0], Signature::default());
    }

    #[test]
    fn builds_threshold_transaction_with_three_required_signers() {
        let fee_payer = Pubkey::new_from_array([23; 32]);
        let settings_authority = Pubkey::new_from_array([21; 32]);
        let rent_payer = Pubkey::new_from_array([22; 32]);
        let recent_blockhash = Hash::new_from_array([30; 32]);

        let transaction = build_change_threshold_as_authority_transaction(
            fee_payer,
            recent_blockhash,
            AuthoritySettingsAccounts {
                settings: Pubkey::new_from_array([20; 32]),
                settings_authority,
                rent_payer: Some(rent_payer),
            },
            change_threshold_args(),
        )
        .unwrap();

        assert_eq!(transaction.message.account_keys[0], fee_payer);
        assert_eq!(transaction.message.header.num_required_signatures, 3);
        assert_eq!(transaction.signatures.len(), 3);

        assert!(
            transaction
                .signatures
                .iter()
                .all(|signature| signature == &Signature::default())
        );

        assert!(
            transaction
                .message
                .account_keys
                .contains(&settings_authority)
        );

        assert!(transaction.message.account_keys.contains(&rent_payer));
    }

    #[test]
    fn serializes_add_signer_data() {
        let instruction = build_add_signer_as_authority_instruction(
            authority_settings_accounts(Some(Pubkey::new_from_array([22; 32]))),
            AddSignerArgs {
                new_signer: SmartAccountSigner {
                    key: Pubkey::new_from_array([31; 32]),
                    permissions: Permissions { mask: 7 },
                },
                memo: Some("add".to_owned()),
            },
        )
        .unwrap();

        let mut expected_data = vec![80, 198, 228, 154, 7, 234, 99, 56];

        // new_signer.key
        expected_data.extend_from_slice(&[31; 32]);

        // new_signer.permissions.mask
        expected_data.push(7);

        // memo: Some("add")
        expected_data.push(1);
        expected_data.extend_from_slice(&3_u32.to_le_bytes());
        expected_data.extend_from_slice(b"add");

        assert_eq!(instruction.data, expected_data);
    }

    #[test]
    fn serializes_remove_signer_data() {
        let instruction = build_remove_signer_as_authority_instruction(
            authority_settings_accounts(None),
            RemoveSignerArgs {
                old_signer: Pubkey::new_from_array([31; 32]),
                memo: None,
            },
        )
        .unwrap();

        let mut expected_data = vec![58, 19, 149, 16, 181, 16, 125, 148];

        // old_signer
        expected_data.extend_from_slice(&[31; 32]);

        // memo: None
        expected_data.push(0);

        assert_eq!(instruction.data, expected_data);
    }

    #[test]
    fn builds_unsigned_add_signer_transaction() {
        let authority = Pubkey::new_from_array([21; 32]);
        let recent_blockhash = Hash::new_from_array([40; 32]);

        let transaction = build_add_signer_as_authority_transaction(
            authority,
            recent_blockhash,
            AuthoritySettingsAccounts {
                settings: Pubkey::new_from_array([20; 32]),
                settings_authority: authority,
                rent_payer: Some(authority),
            },
            AddSignerArgs {
                new_signer: SmartAccountSigner {
                    key: Pubkey::new_from_array([31; 32]),
                    permissions: Permissions { mask: 7 },
                },
                memo: Some("add".to_owned()),
            },
        )
        .unwrap();

        assert_eq!(transaction.message.recent_blockhash, recent_blockhash);
        assert_eq!(transaction.message.account_keys[0], authority);
        assert_eq!(transaction.message.header.num_required_signatures, 1);
        assert_eq!(transaction.signatures.len(), 1);
        assert_eq!(transaction.signatures[0], Signature::default());
    }

    #[test]
    fn builds_unsigned_remove_signer_transaction() {
        let authority = Pubkey::new_from_array([21; 32]);
        let recent_blockhash = Hash::new_from_array([41; 32]);

        let transaction = build_remove_signer_as_authority_transaction(
            authority,
            recent_blockhash,
            AuthoritySettingsAccounts {
                settings: Pubkey::new_from_array([20; 32]),
                settings_authority: authority,
                rent_payer: None,
            },
            RemoveSignerArgs {
                old_signer: Pubkey::new_from_array([31; 32]),
                memo: Some("remove".to_owned()),
            },
        )
        .unwrap();

        assert_eq!(transaction.message.recent_blockhash, recent_blockhash);
        assert_eq!(transaction.message.account_keys[0], authority);
        assert_eq!(transaction.message.header.num_required_signatures, 1);
        assert_eq!(transaction.signatures.len(), 1);
        assert_eq!(transaction.signatures[0], Signature::default());
    }

    #[test]
    fn serializes_set_time_lock_data() {
        let instruction = build_set_time_lock_as_authority_instruction(
            authority_settings_accounts(None),
            SetTimeLockArgs {
                time_lock: 3_600,
                memo: Some("lock".to_owned()),
            },
        )
        .unwrap();

        let mut expected_data = vec![2, 234, 93, 93, 40, 92, 31, 234];

        // time_lock
        expected_data.extend_from_slice(&3_600_u32.to_le_bytes());

        // memo: Some("lock")
        expected_data.push(1);
        expected_data.extend_from_slice(&4_u32.to_le_bytes());
        expected_data.extend_from_slice(b"lock");

        assert_eq!(instruction.data, expected_data);
    }
}

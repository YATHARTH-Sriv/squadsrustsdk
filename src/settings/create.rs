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
    pda::{program_config_pda, settings_pda},
    program_config::ProgramConfig,
    transaction::build_unsigned_transaction,
    types::SmartAccountSigner,
};

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize)]
pub struct CreateSmartAccountArgs {
    pub settings_authority: Option<Pubkey>,
    pub threshold: u16,
    pub signers: Vec<SmartAccountSigner>,
    pub time_lock: u32,
    pub rent_collector: Option<Pubkey>,
    pub memo: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreateSmartAccountAccounts {
    pub treasury: Pubkey,
    pub creator: Pubkey,
    pub settings: Pubkey,
}

pub fn build_create_smart_account_instruction(
    accounts: CreateSmartAccountAccounts,
    args: CreateSmartAccountArgs,
) -> Result<Instruction, SdkError> {
    let discriminator = anchor_discriminator("global:create_smart_account");

    let encoded_args = borsh::to_vec(&args).map_err(SdkError::CreateSmartAccountSerialization)?;

    let mut data = Vec::with_capacity(discriminator.len() + encoded_args.len());

    data.extend_from_slice(&discriminator);
    data.extend_from_slice(&encoded_args);

    let (program_config, _) = program_config_pda();

    let accounts = vec![
        AccountMeta::new(program_config, false),
        AccountMeta::new(accounts.treasury, false),
        AccountMeta::new(accounts.creator, true),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new_readonly(PROGRAM_ID, false),
        AccountMeta::new(accounts.settings, false),
    ];

    Ok(Instruction {
        program_id: PROGRAM_ID,
        accounts,
        data,
    })
}

#[derive(Debug)]
pub struct CreateSmartAccountTransaction {
    pub transaction: Transaction,
    pub settings: Pubkey,
    pub settings_seed: u128,
}

pub fn build_create_smart_account_transaction(
    fee_payer: Pubkey,
    recent_blockhash: Hash,
    program_config: &ProgramConfig,
    creator: Pubkey,
    args: CreateSmartAccountArgs,
) -> Result<CreateSmartAccountTransaction, SdkError> {
    let settings_seed = program_config
        .smart_account_index
        .checked_add(1)
        .ok_or(SdkError::SmartAccountIndexOverflow)?;

    let (settings, _) = settings_pda(settings_seed);

    let instruction = build_create_smart_account_instruction(
        CreateSmartAccountAccounts {
            treasury: program_config.treasury,
            creator,
            settings,
        },
        args,
    )?;

    let transaction = build_unsigned_transaction(fee_payer, recent_blockhash, instruction);

    Ok(CreateSmartAccountTransaction {
        transaction,
        settings,
        settings_seed,
    })
}

#[cfg(test)]
mod tests {
    use solana_sdk::signature::Signature;

    use super::*;
    use crate::types::Permissions;

    fn test_accounts() -> CreateSmartAccountAccounts {
        CreateSmartAccountAccounts {
            treasury: Pubkey::new_from_array([1; 32]),
            creator: Pubkey::new_from_array([2; 32]),
            settings: Pubkey::new_from_array([3; 32]),
        }
    }

    fn test_args() -> CreateSmartAccountArgs {
        CreateSmartAccountArgs {
            settings_authority: Some(Pubkey::new_from_array([4; 32])),
            threshold: 2,
            signers: vec![
                SmartAccountSigner {
                    key: Pubkey::new_from_array([5; 32]),
                    permissions: Permissions { mask: 7 },
                },
                SmartAccountSigner {
                    key: Pubkey::new_from_array([6; 32]),
                    permissions: Permissions { mask: 3 },
                },
            ],
            time_lock: 300,
            rent_collector: Some(Pubkey::new_from_array([7; 32])),
            memo: Some("hello".to_owned()),
        }
    }

    fn test_program_config(smart_account_index: u128) -> ProgramConfig {
        ProgramConfig {
            smart_account_index,
            authority: Pubkey::new_from_array([10; 32]),
            smart_account_creation_fee: 5_000,
            treasury: Pubkey::new_from_array([11; 32]),
            reserved: [0; 64],
        }
    }
    #[test]
    fn builds_create_smart_account_account_metadata() {
        let instruction =
            build_create_smart_account_instruction(test_accounts(), test_args()).unwrap();

        let (program_config, _) = program_config_pda();

        assert_eq!(instruction.program_id, PROGRAM_ID);

        assert_eq!(
            instruction.accounts,
            vec![
                AccountMeta::new(program_config, false),
                AccountMeta::new(Pubkey::new_from_array([1; 32]), false),
                AccountMeta::new(Pubkey::new_from_array([2; 32]), true),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                AccountMeta::new_readonly(PROGRAM_ID, false),
                AccountMeta::new(Pubkey::new_from_array([3; 32]), false),
            ]
        );
    }

    #[test]
    fn serializes_create_smart_account_data() {
        let instruction =
            build_create_smart_account_instruction(test_accounts(), test_args()).unwrap();

        let mut expected_data = vec![197, 102, 253, 231, 77, 84, 50, 17];

        // settings_authority: Some(pubkey)
        expected_data.push(1);
        expected_data.extend_from_slice(&[4; 32]);

        // threshold
        expected_data.extend_from_slice(&2_u16.to_le_bytes());

        // signers vector length
        expected_data.extend_from_slice(&2_u32.to_le_bytes());

        // First signer
        expected_data.extend_from_slice(&[5; 32]);
        expected_data.push(7);

        // Second signer
        expected_data.extend_from_slice(&[6; 32]);
        expected_data.push(3);

        // time_lock
        expected_data.extend_from_slice(&300_u32.to_le_bytes());

        // rent_collector: Some(pubkey)
        expected_data.push(1);
        expected_data.extend_from_slice(&[7; 32]);

        // memo: Some("hello")
        expected_data.push(1);
        expected_data.extend_from_slice(&5_u32.to_le_bytes());
        expected_data.extend_from_slice(b"hello");

        assert_eq!(instruction.data, expected_data);
    }

    #[test]
    fn builds_unsigned_create_smart_account_transaction() {
        let fee_payer = Pubkey::new_from_array([8; 32]);
        let creator = Pubkey::new_from_array([2; 32]);
        let recent_blockhash = Hash::new_from_array([9; 32]);
        let program_config = test_program_config(7);

        let result = build_create_smart_account_transaction(
            fee_payer,
            recent_blockhash,
            &program_config,
            creator,
            test_args(),
        )
        .unwrap();

        let (expected_settings, _) = settings_pda(8);

        assert_eq!(result.settings_seed, 8);
        assert_eq!(result.settings, expected_settings);

        assert_eq!(
            result.transaction.message.recent_blockhash,
            recent_blockhash
        );

        assert_eq!(result.transaction.message.account_keys[0], fee_payer);

        assert!(
            result
                .transaction
                .message
                .account_keys
                .contains(&program_config.treasury)
        );

        assert!(
            result
                .transaction
                .message
                .account_keys
                .contains(&expected_settings)
        );

        // The fee payer and creator are different, so both must sign.
        assert_eq!(result.transaction.message.header.num_required_signatures, 2);

        assert_eq!(result.transaction.signatures.len(), 2);

        assert!(
            result
                .transaction
                .signatures
                .iter()
                .all(|signature| signature == &Signature::default())
        );
    }

    #[test]
    fn rejects_smart_account_index_overflow() {
        let fee_payer = Pubkey::new_from_array([8; 32]);
        let creator = Pubkey::new_from_array([2; 32]);
        let recent_blockhash = Hash::new_from_array([9; 32]);
        let program_config = test_program_config(u128::MAX);

        let result = build_create_smart_account_transaction(
            fee_payer,
            recent_blockhash,
            &program_config,
            creator,
            test_args(),
        );

        assert!(matches!(result, Err(SdkError::SmartAccountIndexOverflow)));
    }
}

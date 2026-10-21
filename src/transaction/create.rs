use borsh::BorshSerialize;
use solana_sdk::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    transaction::Transaction,
};

use crate::{
    SdkError,
    constant::{PROGRAM_ID, SYSTEM_PROGRAM_ID},
    helpers::anchor_discriminator,
    pda::{smart_account_pda, transaction_pda},
    settings::SettingsAccount,
    transaction::{
        build_unsigned_transaction, compile_transaction_message, serialize_transaction_message,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize)]
pub struct CreateTransactionArgs {
    pub account_index: u8,
    pub ephemeral_signers: u8,
    pub transaction_message: Vec<u8>,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreateTransactionAccounts {
    pub consensus_account: Pubkey,
    pub transaction: Pubkey,
    pub creator: Pubkey,
    pub rent_payer: Pubkey,
}

#[derive(BorshSerialize)]
enum CreateTransactionWireArgs {
    TransactionPayload(CreateTransactionArgs),
}

#[derive(Debug)]
pub struct CreateTransactionOutput {
    pub unsigned_transaction: Transaction,
    pub transaction_address: Pubkey,
    pub transaction_index: u64,
    pub smart_account: Pubkey,
}

pub fn build_create_transaction_instruction(
    accounts: CreateTransactionAccounts,
    args: CreateTransactionArgs,
) -> Result<Instruction, SdkError> {
    let discriminator = anchor_discriminator("global:create_transaction");

    let serialized_args = borsh::to_vec(&CreateTransactionWireArgs::TransactionPayload(args))
        .map_err(SdkError::CreateTransactionSerialization)?;

    let mut data = Vec::with_capacity(8 + serialized_args.len());

    data.extend_from_slice(&discriminator);
    data.extend_from_slice(&serialized_args);

    Ok(Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(accounts.consensus_account, false),
            AccountMeta::new(accounts.transaction, false),
            AccountMeta::new_readonly(accounts.creator, true),
            AccountMeta::new(accounts.rent_payer, true),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
        ],
        data,
    })
}

#[derive(Debug)]
pub struct CreateTransactionInput<'a> {
    pub fee_payer: Pubkey,
    pub recent_blockhash: Hash,
    pub settings_address: Pubkey,
    pub settings: &'a SettingsAccount,
    pub creator: Pubkey,
    pub rent_payer: Pubkey,
    pub account_index: u8,
    pub inner_instructions: &'a [Instruction],
    pub memo: Option<String>,
}

pub fn build_create_transaction(
    input: CreateTransactionInput<'_>,
) -> Result<CreateTransactionOutput, SdkError> {
    let transaction_index = input
        .settings
        .transaction_index
        .checked_add(1)
        .ok_or(SdkError::TransactionIndexOverflow)?;

    let (smart_account, _) = smart_account_pda(&input.settings_address, input.account_index);

    let (transaction_address, _) = transaction_pda(&input.settings_address, transaction_index);

    let compiled_message = compile_transaction_message(smart_account, input.inner_instructions)?;

    let transaction_message = serialize_transaction_message(&compiled_message)?;

    let create_instruction = build_create_transaction_instruction(
        CreateTransactionAccounts {
            consensus_account: input.settings_address,
            transaction: transaction_address,
            creator: input.creator,
            rent_payer: input.rent_payer,
        },
        CreateTransactionArgs {
            account_index: input.account_index,
            ephemeral_signers: 0,
            transaction_message,
            memo: input.memo,
        },
    )?;

    let unsigned_transaction =
        build_unsigned_transaction(input.fee_payer, input.recent_blockhash, create_instruction);

    Ok(CreateTransactionOutput {
        unsigned_transaction,
        transaction_address,
        transaction_index,
        smart_account,
    })
}

#[cfg(test)]
mod tests {
    use solana_sdk::{
        hash::Hash,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Signature,
    };

    use super::{
        CreateTransactionAccounts, CreateTransactionArgs, CreateTransactionInput,
        build_create_transaction, build_create_transaction_instruction,
    };

    use crate::{
        SdkError,
        constant::{PROGRAM_ID, SYSTEM_PROGRAM_ID},
        helpers::anchor_discriminator,
        pda::{smart_account_pda, transaction_pda},
        settings::SettingsAccount,
    };

    #[test]
    fn builds_create_transaction_account_metadata() {
        let consensus_account = Pubkey::new_from_array([1; 32]);
        let transaction = Pubkey::new_from_array([2; 32]);
        let creator = Pubkey::new_from_array([3; 32]);
        let rent_payer = Pubkey::new_from_array([4; 32]);

        let instruction = build_create_transaction_instruction(
            CreateTransactionAccounts {
                consensus_account,
                transaction,
                creator,
                rent_payer,
            },
            CreateTransactionArgs {
                account_index: 0,
                ephemeral_signers: 0,
                transaction_message: vec![],
                memo: None,
            },
        )
        .unwrap();

        assert_eq!(instruction.program_id, PROGRAM_ID);

        assert_eq!(
            instruction.accounts,
            vec![
                AccountMeta::new(consensus_account, false),
                AccountMeta::new(transaction, false),
                AccountMeta::new_readonly(creator, true),
                AccountMeta::new(rent_payer, true),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                AccountMeta::new_readonly(PROGRAM_ID, false),
            ]
        );
    }

    #[test]
    fn serializes_create_transaction_instruction_data() {
        let instruction = build_create_transaction_instruction(
            CreateTransactionAccounts {
                consensus_account: Pubkey::new_from_array([1; 32]),
                transaction: Pubkey::new_from_array([2; 32]),
                creator: Pubkey::new_from_array([3; 32]),
                rent_payer: Pubkey::new_from_array([4; 32]),
            },
            CreateTransactionArgs {
                account_index: 4,
                ephemeral_signers: 0,
                transaction_message: vec![10, 20, 30],
                memo: Some("hello".to_string()),
            },
        )
        .unwrap();

        let mut expected_data = anchor_discriminator("global:create_transaction").to_vec();

        // CreateTransactionWireArgs::TransactionPayload is enum variant 0.
        expected_data.push(0);

        expected_data.push(4); // account_index
        expected_data.push(0); // ephemeral_signers

        // Borsh Vec<u8>: u32 length followed by the bytes.
        expected_data.extend_from_slice(&3_u32.to_le_bytes());
        expected_data.extend_from_slice(&[10, 20, 30]);

        // Borsh Option::Some uses tag 1.
        expected_data.push(1);

        // Borsh String: u32 byte length followed by UTF-8 bytes.
        expected_data.extend_from_slice(&5_u32.to_le_bytes());
        expected_data.extend_from_slice(b"hello");

        assert_eq!(instruction.data, expected_data);
    }

    fn test_settings(transaction_index: u64) -> SettingsAccount {
        SettingsAccount {
            seed: 1,
            settings_authority: Pubkey::new_from_array([20; 32]),
            threshold: 2,
            time_lock: 0,
            transaction_index,
            stale_transaction_index: 0,
            archival_authority: None,
            archivable_after: 0,
            bump: 250,
            signers: vec![],
            account_utilization: 1,
            policy_seed: None,
            reserved2: 0,
        }
    }

    #[test]
    fn builds_complete_unsigned_create_transaction() {
        let fee_payer = Pubkey::new_from_array([30; 32]);
        let creator = Pubkey::new_from_array([31; 32]);
        let rent_payer = Pubkey::new_from_array([32; 32]);
        let settings_address = Pubkey::new_from_array([33; 32]);
        let recent_blockhash = Hash::new_from_array([34; 32]);

        let settings = test_settings(7);
        let account_index = 0;

        let (expected_smart_account, _) = smart_account_pda(&settings_address, account_index);

        let recipient = Pubkey::new_from_array([35; 32]);
        let inner_program = Pubkey::new_from_array([36; 32]);

        let inner_instructions = vec![Instruction {
            program_id: inner_program,
            accounts: vec![
                AccountMeta::new(expected_smart_account, true),
                AccountMeta::new(recipient, false),
            ],
            data: vec![10, 20, 30],
        }];

        let output = build_create_transaction(CreateTransactionInput {
            fee_payer,
            recent_blockhash,
            settings_address,
            settings: &settings,
            creator,
            rent_payer,
            account_index,
            inner_instructions: &inner_instructions,
            memo: Some("test transaction".to_string()),
        })
        .unwrap();

        let expected_transaction_index = 8;

        let (expected_transaction_address, _) =
            transaction_pda(&settings_address, expected_transaction_index);

        assert_eq!(output.transaction_index, expected_transaction_index);
        assert_eq!(output.transaction_address, expected_transaction_address);
        assert_eq!(output.smart_account, expected_smart_account);

        let message = &output.unsigned_transaction.message;

        assert_eq!(message.recent_blockhash, recent_blockhash);
        assert_eq!(message.account_keys[0], fee_payer);

        assert!(message.account_keys.contains(&settings_address));
        assert!(message.account_keys.contains(&expected_transaction_address));
        assert!(message.account_keys.contains(&creator));
        assert!(message.account_keys.contains(&rent_payer));

        // fee_payer, creator and rent_payer are three different signers.
        assert_eq!(message.header.num_required_signatures, 3);
        assert_eq!(output.unsigned_transaction.signatures.len(), 3);

        assert!(
            output
                .unsigned_transaction
                .signatures
                .iter()
                .all(|signature| signature == &Signature::default())
        );

        assert_eq!(message.instructions.len(), 1);

        let create_instruction = &message.instructions[0];

        let create_program = message.account_keys[usize::from(create_instruction.program_id_index)];

        assert_eq!(create_program, PROGRAM_ID);

        assert_eq!(
            &create_instruction.data[..8],
            &anchor_discriminator("global:create_transaction")
        );

        assert_eq!(create_instruction.data[8], 0); // TransactionPayload variant
        assert_eq!(create_instruction.data[9], account_index);
        assert_eq!(create_instruction.data[10], 0); // ephemeral_signers
    }

    #[test]
    fn rejects_transaction_index_overflow() {
        let settings = test_settings(u64::MAX);
        let inner_instructions = vec![];

        let result = build_create_transaction(CreateTransactionInput {
            fee_payer: Pubkey::new_from_array([30; 32]),
            recent_blockhash: Hash::new_from_array([34; 32]),
            settings_address: Pubkey::new_from_array([33; 32]),
            settings: &settings,
            creator: Pubkey::new_from_array([31; 32]),
            rent_payer: Pubkey::new_from_array([32; 32]),
            account_index: 0,
            inner_instructions: &inner_instructions,
            memo: None,
        });

        assert!(matches!(result, Err(SdkError::TransactionIndexOverflow)));
    }
}

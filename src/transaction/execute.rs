use solana_sdk::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    transaction::Transaction,
};

use super::{TransactionAccount, build_unsigned_transaction};

use crate::{
    SdkError,
    constant::PROGRAM_ID,
    helpers::anchor_discriminator,
    pda::{proposal_pda, smart_account_pda, transaction_pda},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecuteTransactionAccounts {
    pub consensus_account: Pubkey,
    pub proposal: Pubkey,
    pub transaction: Pubkey,
    pub signer: Pubkey,
}

#[derive(Debug)]
pub struct ExecuteTransactionInput<'a> {
    pub fee_payer: Pubkey,
    pub recent_blockhash: Hash,
    pub signer: Pubkey,
    pub transaction: &'a TransactionAccount,
}

#[derive(Debug)]
pub struct ExecuteTransactionOutput {
    pub unsigned_transaction: Transaction,
    pub proposal_address: Pubkey,
    pub transaction_address: Pubkey,
    pub transaction_index: u64,
}

pub fn build_execute_transaction_instruction(
    accounts: ExecuteTransactionAccounts,
    mut remaining_accounts: Vec<AccountMeta>,
) -> Instruction {
    let mut account_metas = vec![
        AccountMeta::new(accounts.consensus_account, false),
        AccountMeta::new(accounts.proposal, false),
        AccountMeta::new_readonly(accounts.transaction, false),
        AccountMeta::new_readonly(accounts.signer, true),
        AccountMeta::new_readonly(PROGRAM_ID, false),
    ];

    account_metas.append(&mut remaining_accounts);

    Instruction {
        program_id: PROGRAM_ID,
        accounts: account_metas,
        data: anchor_discriminator("global:execute_transaction").to_vec(),
    }
}

pub fn build_execute_transaction_remaining_accounts(
    transaction: &TransactionAccount,
) -> Result<Vec<AccountMeta>, SdkError> {
    let payload = &transaction.payload;
    let message = &payload.message;

    if !message.address_table_lookups.is_empty() {
        return Err(SdkError::UnsupportedExecutionAddressLookupTables);
    }

    if !payload.ephemeral_signer_bumps.is_empty() {
        return Err(SdkError::UnsupportedExecutionEphemeralSigners);
    }

    let account_count = message.account_keys.len();

    let num_signers = usize::from(message.num_signers);

    let num_writable_signers = usize::from(message.num_writable_signers);

    let num_writable_non_signers = usize::from(message.num_writable_non_signers);

    let writable_non_signers_end = num_signers
        .checked_add(num_writable_non_signers)
        .ok_or(SdkError::InvalidTransactionMessageHeader)?;

    if num_writable_signers > num_signers
        || num_signers > account_count
        || writable_non_signers_end > account_count
    {
        return Err(SdkError::InvalidTransactionMessageHeader);
    }

    let (smart_account, _) =
        smart_account_pda(&transaction.consensus_account, payload.account_index);

    let remaining_accounts = message
        .account_keys
        .iter()
        .enumerate()
        .map(|(account_index, account_key)| {
            let is_writable_signer = account_index < num_writable_signers;

            let is_writable_non_signer =
                account_index >= num_signers && account_index < writable_non_signers_end;

            let is_writable = is_writable_signer || is_writable_non_signer;

            let is_message_signer = account_index < num_signers;

            // The smart account is a PDA. The onchain program
            // signs for it with invoke_signed.
            let is_signer = is_message_signer && account_key != &smart_account;

            AccountMeta {
                pubkey: *account_key,
                is_signer,
                is_writable,
            }
        })
        .collect();

    Ok(remaining_accounts)
}

pub fn build_execute_transaction(
    input: ExecuteTransactionInput<'_>,
) -> Result<ExecuteTransactionOutput, SdkError> {
    let consensus_account = input.transaction.consensus_account;

    let transaction_index = input.transaction.index;

    let (proposal_address, _) = proposal_pda(&consensus_account, transaction_index);

    let (transaction_address, _) = transaction_pda(&consensus_account, transaction_index);

    let remaining_accounts = build_execute_transaction_remaining_accounts(input.transaction)?;

    let instruction = build_execute_transaction_instruction(
        ExecuteTransactionAccounts {
            consensus_account,
            proposal: proposal_address,
            transaction: transaction_address,
            signer: input.signer,
        },
        remaining_accounts,
    );

    let unsigned_transaction =
        build_unsigned_transaction(input.fee_payer, input.recent_blockhash, instruction);

    Ok(ExecuteTransactionOutput {
        unsigned_transaction,
        proposal_address,
        transaction_address,
        transaction_index,
    })
}

#[cfg(test)]
mod tests {
    use solana_sdk::{hash::Hash, instruction::AccountMeta, pubkey::Pubkey, signature::Signature};

    use super::{
        ExecuteTransactionAccounts, ExecuteTransactionInput, build_execute_transaction,
        build_execute_transaction_instruction, build_execute_transaction_remaining_accounts,
    };

    use crate::{
        SdkError,
        constant::PROGRAM_ID,
        helpers::anchor_discriminator,
        pda::{proposal_pda, smart_account_pda, transaction_pda},
        transaction::{
            SmartAccountMessageAddressTableLookup, SmartAccountTransactionMessage,
            TransactionAccount, TransactionPayloadDetails,
        },
    };

    fn test_transaction() -> TransactionAccount {
        let consensus_account = Pubkey::new_from_array([1; 32]);

        let account_index = 0;

        let (smart_account, _) = smart_account_pda(&consensus_account, account_index);

        TransactionAccount {
            consensus_account,
            creator: Pubkey::new_from_array([2; 32]),
            rent_collector: Pubkey::new_from_array([3; 32]),
            index: 7,
            payload: TransactionPayloadDetails {
                account_index,
                ephemeral_signer_bumps: vec![],
                message: SmartAccountTransactionMessage {
                    num_signers: 2,
                    num_writable_signers: 1,
                    num_writable_non_signers: 2,
                    account_keys: vec![
                        // Writable PDA signer
                        smart_account,
                        // Readonly external signer
                        Pubkey::new_from_array([4; 32]),
                        // Writable non-signers
                        Pubkey::new_from_array([5; 32]),
                        Pubkey::new_from_array([6; 32]),
                        // Readonly non-signer
                        Pubkey::new_from_array([7; 32]),
                    ],
                    instructions: vec![],
                    address_table_lookups: vec![],
                },
            },
        }
    }

    #[test]
    fn reconstructs_execution_account_metadata() {
        let transaction = test_transaction();

        let accounts = build_execute_transaction_remaining_accounts(&transaction).unwrap();

        let (smart_account, _) = smart_account_pda(
            &transaction.consensus_account,
            transaction.payload.account_index,
        );

        assert_eq!(
            accounts,
            vec![
                // Smart account remains writable, but the outer
                // transaction must not require its signature.
                AccountMeta::new(smart_account, false),
                // External readonly signer must sign the outer
                // execution transaction.
                AccountMeta::new_readonly(Pubkey::new_from_array([4; 32]), true,),
                AccountMeta::new(Pubkey::new_from_array([5; 32]), false,),
                AccountMeta::new(Pubkey::new_from_array([6; 32]), false,),
                AccountMeta::new_readonly(Pubkey::new_from_array([7; 32]), false,),
            ]
        );
    }

    #[test]
    fn rejects_invalid_transaction_message_header() {
        let mut transaction = test_transaction();

        transaction.payload.message.num_writable_signers = 3;

        let result = build_execute_transaction_remaining_accounts(&transaction);

        assert!(matches!(
            result,
            Err(SdkError::InvalidTransactionMessageHeader)
        ));
    }

    #[test]
    fn rejects_execution_address_lookup_tables() {
        let mut transaction = test_transaction();

        transaction.payload.message.address_table_lookups.push(
            SmartAccountMessageAddressTableLookup {
                account_key: Pubkey::new_from_array([8; 32]),
                writable_indexes: vec![0],
                readonly_indexes: vec![1],
            },
        );

        let result = build_execute_transaction_remaining_accounts(&transaction);

        assert!(matches!(
            result,
            Err(SdkError::UnsupportedExecutionAddressLookupTables)
        ));
    }

    #[test]
    fn rejects_execution_ephemeral_signers() {
        let mut transaction = test_transaction();

        transaction.payload.ephemeral_signer_bumps.push(250);

        let result = build_execute_transaction_remaining_accounts(&transaction);

        assert!(matches!(
            result,
            Err(SdkError::UnsupportedExecutionEphemeralSigners)
        ));
    }

    #[test]
    fn builds_execute_transaction_account_metadata() {
        let accounts = ExecuteTransactionAccounts {
            consensus_account: Pubkey::new_from_array([10; 32]),
            proposal: Pubkey::new_from_array([11; 32]),
            transaction: Pubkey::new_from_array([12; 32]),
            signer: Pubkey::new_from_array([13; 32]),
        };

        let remaining_accounts = vec![
            AccountMeta::new_readonly(Pubkey::new_from_array([14; 32]), false),
            AccountMeta::new(Pubkey::new_from_array([15; 32]), true),
        ];

        let instruction = build_execute_transaction_instruction(accounts, remaining_accounts);

        assert_eq!(instruction.program_id, PROGRAM_ID);

        assert_eq!(
            instruction.accounts,
            vec![
                AccountMeta::new(accounts.consensus_account, false,),
                AccountMeta::new(accounts.proposal, false),
                AccountMeta::new_readonly(accounts.transaction, false,),
                AccountMeta::new_readonly(accounts.signer, true,),
                AccountMeta::new_readonly(PROGRAM_ID, false),
                // Dynamic accounts preserve their order and flags.
                AccountMeta::new_readonly(Pubkey::new_from_array([14; 32]), false,),
                AccountMeta::new(Pubkey::new_from_array([15; 32]), true,),
            ]
        );
    }

    #[test]
    fn serializes_execute_transaction_discriminator() {
        let instruction = build_execute_transaction_instruction(
            ExecuteTransactionAccounts {
                consensus_account: Pubkey::new_from_array([10; 32]),
                proposal: Pubkey::new_from_array([11; 32]),
                transaction: Pubkey::new_from_array([12; 32]),
                signer: Pubkey::new_from_array([13; 32]),
            },
            vec![],
        );

        assert_eq!(instruction.data.len(), 8);

        assert_eq!(instruction.data, vec![231, 173, 49, 91, 235, 24, 68, 19]);
    }

    #[test]
    fn builds_complete_unsigned_execute_transaction() {
        let transaction_account = test_transaction();

        let fee_payer = Pubkey::new_from_array([20; 32]);
        let signer = Pubkey::new_from_array([21; 32]);
        let recent_blockhash = Hash::new_from_array([22; 32]);

        let output = build_execute_transaction(ExecuteTransactionInput {
            fee_payer,
            recent_blockhash,
            signer,
            transaction: &transaction_account,
        })
        .unwrap();

        let consensus_account = transaction_account.consensus_account;

        let transaction_index = transaction_account.index;

        let (expected_proposal, _) = proposal_pda(&consensus_account, transaction_index);

        let (expected_transaction, _) = transaction_pda(&consensus_account, transaction_index);

        assert_eq!(output.proposal_address, expected_proposal);

        assert_eq!(output.transaction_address, expected_transaction);

        assert_eq!(output.transaction_index, transaction_index);

        let transaction = &output.unsigned_transaction;
        let message = &transaction.message;

        assert_eq!(message.recent_blockhash, recent_blockhash);
        assert_eq!(message.account_keys[0], fee_payer);

        assert!(message.account_keys.contains(&consensus_account));
        assert!(message.account_keys.contains(&expected_proposal));
        assert!(message.account_keys.contains(&expected_transaction));
        assert!(message.account_keys.contains(&signer));

        // The stored message contains external signer [4; 32].
        assert!(
            message
                .account_keys
                .contains(&Pubkey::new_from_array([4; 32]))
        );

        // Required signatures:
        // 1. fee payer
        // 2. execution signer
        // 3. external signer from the stored transaction message
        assert_eq!(message.header.num_required_signatures, 3);
        assert_eq!(transaction.signatures.len(), 3);

        assert!(
            transaction
                .signatures
                .iter()
                .all(|signature| signature == &Signature::default())
        );

        assert_eq!(message.instructions.len(), 1);

        let execute_instruction = &message.instructions[0];

        let instruction_program =
            message.account_keys[usize::from(execute_instruction.program_id_index)];

        assert_eq!(instruction_program, PROGRAM_ID);

        assert_eq!(
            execute_instruction.data,
            anchor_discriminator("global:execute_transaction")
        );
    }

    #[test]
    fn high_level_execute_rejects_unsupported_lookups() {
        let mut transaction_account = test_transaction();

        transaction_account
            .payload
            .message
            .address_table_lookups
            .push(SmartAccountMessageAddressTableLookup {
                account_key: Pubkey::new_from_array([30; 32]),
                writable_indexes: vec![0],
                readonly_indexes: vec![],
            });

        let result = build_execute_transaction(ExecuteTransactionInput {
            fee_payer: Pubkey::new_from_array([20; 32]),
            recent_blockhash: Hash::new_from_array([22; 32]),
            signer: Pubkey::new_from_array([21; 32]),
            transaction: &transaction_account,
        });

        assert!(matches!(
            result,
            Err(SdkError::UnsupportedExecutionAddressLookupTables)
        ));
    }
}

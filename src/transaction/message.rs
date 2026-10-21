use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::{
    hash::Hash,
    message::{Instruction, v0},
    pubkey::Pubkey,
};

use crate::SdkError;

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct SmartAccountCompiledInstruction {
    pub program_id_index: u8,
    pub account_indexes: Vec<u8>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct SmartAccountMessageAddressTableLookup {
    pub account_key: Pubkey,
    pub writable_indexes: Vec<u8>,
    pub readonly_indexes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct SmartAccountTransactionMessage {
    pub num_signers: u8,
    pub num_writable_signers: u8,
    pub num_writable_non_signers: u8,
    pub account_keys: Vec<Pubkey>,
    pub instructions: Vec<SmartAccountCompiledInstruction>,
    pub address_table_lookups: Vec<SmartAccountMessageAddressTableLookup>,
}

pub fn compile_transaction_message(
    smart_account: Pubkey,
    instructions: &[Instruction],
) -> Result<SmartAccountTransactionMessage, SdkError> {
    let compiled_message =
        v0::Message::try_compile(&smart_account, instructions, &[], Hash::default())
            .map_err(SdkError::TransactionMessageCompilation)?;

    let num_signers = compiled_message.header.num_required_signatures;

    let num_writable_signers =
        num_signers.saturating_sub(compiled_message.header.num_readonly_signed_accounts);

    let writable_non_signers = compiled_message
        .account_keys
        .len()
        .saturating_sub(usize::from(num_signers))
        .saturating_sub(usize::from(
            compiled_message.header.num_readonly_unsigned_accounts,
        ));

    let num_writable_non_signers = u8::try_from(writable_non_signers)
        .map_err(|_| SdkError::TransactionMessageAccountCountOverflow)?;

    let instructions = compiled_message
        .instructions
        .into_iter()
        .map(|instruction| SmartAccountCompiledInstruction {
            program_id_index: instruction.program_id_index,
            account_indexes: instruction.accounts,
            data: instruction.data,
        })
        .collect();

    let address_table_lookups = compiled_message
        .address_table_lookups
        .into_iter()
        .map(|lookup| SmartAccountMessageAddressTableLookup {
            account_key: lookup.account_key,
            writable_indexes: lookup.writable_indexes,
            readonly_indexes: lookup.readonly_indexes,
        })
        .collect();

    Ok(SmartAccountTransactionMessage {
        num_signers,
        num_writable_signers,
        num_writable_non_signers,
        account_keys: compiled_message.account_keys,
        instructions,
        address_table_lookups,
    })
}

fn length_as_u8(field: &'static str, length: usize) -> Result<u8, SdkError> {
    u8::try_from(length).map_err(|_| SdkError::TransactionMessageLengthOverflow {
        field,
        actual: length,
        maximum: usize::from(u8::MAX),
    })
}

fn length_as_u16(field: &'static str, length: usize) -> Result<u16, SdkError> {
    u16::try_from(length).map_err(|_| SdkError::TransactionMessageLengthOverflow {
        field,
        actual: length,
        maximum: usize::from(u16::MAX),
    })
}

pub fn serialize_transaction_message(
    message: &SmartAccountTransactionMessage,
) -> Result<Vec<u8>, SdkError> {
    let mut bytes = Vec::new();

    bytes.push(message.num_signers);
    bytes.push(message.num_writable_signers);
    bytes.push(message.num_writable_non_signers);

    let account_keys_length = length_as_u8("transaction account keys", message.account_keys.len())?;

    bytes.push(account_keys_length);

    for account_key in &message.account_keys {
        bytes.extend_from_slice(account_key.as_ref());
    }

    let instructions_length = length_as_u8("transaction instructions", message.instructions.len())?;

    bytes.push(instructions_length);

    for instruction in &message.instructions {
        bytes.push(instruction.program_id_index);

        let account_indexes_length = length_as_u8(
            "compiled instruction account indexes",
            instruction.account_indexes.len(),
        )?;

        bytes.push(account_indexes_length);
        bytes.extend_from_slice(&instruction.account_indexes);

        let instruction_data_length =
            length_as_u16("compiled instruction data", instruction.data.len())?;

        bytes.extend_from_slice(&instruction_data_length.to_le_bytes());

        bytes.extend_from_slice(&instruction.data);
    }

    let lookups_length =
        length_as_u8("address table lookups", message.address_table_lookups.len())?;

    bytes.push(lookups_length);

    for lookup in &message.address_table_lookups {
        bytes.extend_from_slice(lookup.account_key.as_ref());

        let writable_indexes_length =
            length_as_u8("lookup writable indexes", lookup.writable_indexes.len())?;

        bytes.push(writable_indexes_length);
        bytes.extend_from_slice(&lookup.writable_indexes);

        let readonly_indexes_length =
            length_as_u8("lookup readonly indexes", lookup.readonly_indexes.len())?;

        bytes.push(readonly_indexes_length);
        bytes.extend_from_slice(&lookup.readonly_indexes);
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use borsh::BorshDeserialize;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    };

    use super::{
        SmartAccountCompiledInstruction, SmartAccountMessageAddressTableLookup,
        SmartAccountTransactionMessage, compile_transaction_message, serialize_transaction_message,
    };
    use crate::error::SdkError;

    fn account_index(message: &SmartAccountTransactionMessage, key: &Pubkey) -> u8 {
        let index = message
            .account_keys
            .iter()
            .position(|account_key| account_key == key)
            .expect("account key must exist in compiled message");

        u8::try_from(index).expect("compiled account index must fit inside u8")
    }

    #[test]
    fn compiles_instruction_into_smart_account_message() {
        let smart_account = Pubkey::new_from_array([1; 32]);
        let recipient = Pubkey::new_from_array([2; 32]);
        let program_id = Pubkey::new_from_array([3; 32]);
        let readonly_account = Pubkey::new_from_array([4; 32]);

        let instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(smart_account, true),
                AccountMeta::new(recipient, false),
                AccountMeta::new_readonly(readonly_account, false),
            ],
            data: vec![10, 20, 30],
        };

        let message = compile_transaction_message(smart_account, &[instruction]).unwrap();

        assert_eq!(message.num_signers, 1);
        assert_eq!(message.num_writable_signers, 1);
        assert_eq!(message.num_writable_non_signers, 1);

        // The payer is always the first static account.
        assert_eq!(message.account_keys[0], smart_account);

        assert_eq!(message.instructions.len(), 1);
        assert!(message.address_table_lookups.is_empty());

        let compiled_instruction = &message.instructions[0];

        assert_eq!(
            compiled_instruction.program_id_index,
            account_index(&message, &program_id)
        );

        assert_eq!(
            compiled_instruction.account_indexes,
            vec![
                account_index(&message, &smart_account),
                account_index(&message, &recipient),
                account_index(&message, &readonly_account),
            ]
        );

        assert_eq!(compiled_instruction.data, vec![10, 20, 30]);
    }

    #[test]
    fn deduplicates_accounts_and_merges_permissions() {
        let smart_account = Pubkey::new_from_array([1; 32]);
        let shared_account = Pubkey::new_from_array([2; 32]);
        let additional_signer = Pubkey::new_from_array([3; 32]);
        let first_program = Pubkey::new_from_array([4; 32]);
        let second_program = Pubkey::new_from_array([5; 32]);

        let first_instruction = Instruction {
            program_id: first_program,
            accounts: vec![
                AccountMeta::new_readonly(shared_account, false),
                AccountMeta::new_readonly(additional_signer, true),
            ],
            data: vec![1],
        };

        let second_instruction = Instruction {
            program_id: second_program,
            accounts: vec![AccountMeta::new(shared_account, false)],
            data: vec![2],
        };

        let message =
            compile_transaction_message(smart_account, &[first_instruction, second_instruction])
                .unwrap();

        assert_eq!(message.instructions.len(), 2);

        // Smart account payer + additional signer.
        assert_eq!(message.num_signers, 2);

        // Only the smart account payer is a writable signer.
        assert_eq!(message.num_writable_signers, 1);

        // shared_account became writable because the second
        // instruction requested writable access.
        assert_eq!(message.num_writable_non_signers, 1);

        let shared_account_occurrences = message
            .account_keys
            .iter()
            .filter(|key| **key == shared_account)
            .count();

        assert_eq!(shared_account_occurrences, 1);

        let first_compiled = &message.instructions[0];
        let second_compiled = &message.instructions[1];

        assert_eq!(first_compiled.data, vec![1]);
        assert_eq!(second_compiled.data, vec![2]);

        assert_eq!(
            first_compiled.account_indexes,
            vec![
                account_index(&message, &shared_account),
                account_index(&message, &additional_signer),
            ]
        );

        assert_eq!(
            second_compiled.account_indexes,
            vec![account_index(&message, &shared_account)]
        );
    }

    #[test]
    fn compiled_message_survives_borsh_round_trip() {
        let smart_account = Pubkey::new_from_array([1; 32]);
        let recipient = Pubkey::new_from_array([2; 32]);
        let program_id = Pubkey::new_from_array([3; 32]);

        let instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(smart_account, true),
                AccountMeta::new(recipient, false),
            ],
            data: vec![50, 60, 70],
        };

        let original = compile_transaction_message(smart_account, &[instruction]).unwrap();

        let encoded = borsh::to_vec(&original).unwrap();

        let decoded = SmartAccountTransactionMessage::try_from_slice(&encoded).unwrap();

        assert_eq!(decoded, original);
    }

    #[test]
    fn returns_error_when_message_compilation_overflows() {
        let smart_account = Pubkey::new_from_array([255; 32]);

        // 255 unique instruction signers plus the smart account
        // payer results in 256 required signers. The Solana
        // message header stores this count as a u8.
        let accounts = (0_u8..=254)
            .map(|value| AccountMeta::new_readonly(Pubkey::new_from_array([value; 32]), true))
            .collect();

        let instruction = Instruction {
            program_id: Pubkey::new_from_array([0; 32]),
            accounts,
            data: vec![],
        };

        let result = compile_transaction_message(smart_account, &[instruction]);

        assert!(matches!(
            result,
            Err(SdkError::TransactionMessageCompilation(_))
        ));
    }

    fn sample_compact_message() -> SmartAccountTransactionMessage {
        SmartAccountTransactionMessage {
            num_signers: 1,
            num_writable_signers: 1,
            num_writable_non_signers: 1,

            account_keys: vec![
                Pubkey::new_from_array([1; 32]),
                Pubkey::new_from_array([2; 32]),
            ],

            instructions: vec![SmartAccountCompiledInstruction {
                program_id_index: 1,
                account_indexes: vec![0],
                data: vec![10, 20, 30],
            }],

            address_table_lookups: vec![SmartAccountMessageAddressTableLookup {
                account_key: Pubkey::new_from_array([3; 32]),
                writable_indexes: vec![4, 5],
                readonly_indexes: vec![6],
            }],
        }
    }

    #[test]
    fn serializes_compact_transaction_message_bytes() {
        let message = sample_compact_message();

        let actual = serialize_transaction_message(&message).unwrap();

        let mut expected = vec![
            // Message header
            1, // num_signers
            1, // num_writable_signers
            1, // num_writable_non_signers
            // account_keys length as u8
            2,
        ];

        expected.extend_from_slice(&[1; 32]);
        expected.extend_from_slice(&[2; 32]);

        // instructions length as u8
        expected.push(1);

        // First compiled instruction
        expected.push(1); // program_id_index

        // account_indexes length as u8
        expected.push(1);
        expected.push(0);

        // instruction data length as little endian u16
        expected.extend_from_slice(&3_u16.to_le_bytes());
        expected.extend_from_slice(&[10, 20, 30]);

        // address_table_lookups length as u8
        expected.push(1);

        // Lookup table account key
        expected.extend_from_slice(&[3; 32]);

        // writable_indexes length and values
        expected.push(2);
        expected.extend_from_slice(&[4, 5]);

        // readonly_indexes length and values
        expected.push(1);
        expected.push(6);

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_too_many_compact_account_keys() {
        let mut message = sample_compact_message();

        message.account_keys = vec![Pubkey::new_from_array([9; 32]); usize::from(u8::MAX) + 1];

        let result = serialize_transaction_message(&message);

        assert!(matches!(
            result,
            Err(SdkError::TransactionMessageLengthOverflow {
                field: "transaction account keys",
                actual: 256,
                maximum: 255,
            })
        ));
    }

    #[test]
    fn rejects_oversized_compiled_instruction_data() {
        let mut message = sample_compact_message();

        message.instructions[0].data = vec![0; usize::from(u16::MAX) + 1];

        let result = serialize_transaction_message(&message);

        assert!(matches!(
            result,
            Err(SdkError::TransactionMessageLengthOverflow {
                field: "compiled instruction data",
                actual: 65_536,
                maximum: 65_535,
            })
        ));
    }
}

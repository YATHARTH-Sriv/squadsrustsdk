use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;

use crate::{SdkError, constant::PROGRAM_ID, helpers::anchor_discriminator};

use super::SmartAccountTransactionMessage;

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct TransactionPayloadDetails {
    pub account_index: u8,
    pub ephemeral_signer_bumps: Vec<u8>,
    pub message: SmartAccountTransactionMessage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionAccount {
    pub consensus_account: Pubkey,
    pub creator: Pubkey,
    pub rent_collector: Pubkey,
    pub index: u64,
    pub payload: TransactionPayloadDetails,
}

fn deserialize_field<T: BorshDeserialize>(account_data: &mut &[u8]) -> Result<T, SdkError> {
    T::deserialize(account_data).map_err(SdkError::InvalidTransactionData)
}

pub fn decode_transaction_account(
    owner: &Pubkey,
    data: &[u8],
) -> Result<TransactionAccount, SdkError> {
    if owner != &PROGRAM_ID {
        return Err(SdkError::WrongTransactionOwner);
    }

    if data.len() < 8 {
        return Err(SdkError::TransactionDataTooShort);
    }

    let expected_discriminator = anchor_discriminator("account:Transaction");

    if data[..8] != expected_discriminator {
        return Err(SdkError::InvalidTransactionDiscriminator);
    }

    let mut account_data = &data[8..];

    let consensus_account = deserialize_field::<Pubkey>(&mut account_data)?;

    let creator = deserialize_field::<Pubkey>(&mut account_data)?;

    let rent_collector = deserialize_field::<Pubkey>(&mut account_data)?;

    let index = deserialize_field::<u64>(&mut account_data)?;

    let payload_variant = deserialize_field::<u8>(&mut account_data)?;

    let payload = match payload_variant {
        0 => deserialize_field::<TransactionPayloadDetails>(&mut account_data)?,

        1 => return Err(SdkError::UnsupportedPolicyPayload),

        variant => {
            return Err(SdkError::InvalidTransactionPayloadVariant(variant));
        }
    };

    Ok(TransactionAccount {
        consensus_account,
        creator,
        rent_collector,
        index,
        payload,
    })
}

#[cfg(test)]
mod tests {
    use solana_sdk::pubkey::Pubkey;

    use super::decode_transaction_account;
    use crate::{constant::PROGRAM_ID, error::SdkError, helpers::anchor_discriminator};

    fn valid_transaction_data() -> Vec<u8> {
        let mut data = anchor_discriminator("account:Transaction").to_vec();

        // TransactionAccount.consensus_account
        data.extend_from_slice(&[1; 32]);

        // TransactionAccount.creator
        data.extend_from_slice(&[2; 32]);

        // TransactionAccount.rent_collector
        data.extend_from_slice(&[3; 32]);

        // TransactionAccount.index
        data.extend_from_slice(&7_u64.to_le_bytes());

        // Payload::TransactionPayload enum variant
        data.push(0);

        // TransactionPayloadDetails.account_index
        data.push(1);

        // TransactionPayloadDetails.ephemeral_signer_bumps
        // Vec length = 2
        data.extend_from_slice(&2_u32.to_le_bytes());
        data.extend_from_slice(&[9, 10]);

        // SmartAccountTransactionMessage.num_signers
        data.push(1);

        // num_writable_signers
        data.push(1);

        // num_writable_non_signers
        data.push(1);

        // account_keys Vec length = 3
        data.extend_from_slice(&3_u32.to_le_bytes());

        data.extend_from_slice(&[4; 32]);
        data.extend_from_slice(&[5; 32]);
        data.extend_from_slice(&[6; 32]);

        // instructions Vec length = 1
        data.extend_from_slice(&1_u32.to_le_bytes());

        // SmartAccountCompiledInstruction.program_id_index
        data.push(2);

        // account_indexes Vec length = 2
        data.extend_from_slice(&2_u32.to_le_bytes());
        data.extend_from_slice(&[0, 1]);

        // instruction data Vec length = 3
        data.extend_from_slice(&3_u32.to_le_bytes());
        data.extend_from_slice(&[10, 20, 30]);

        // address_table_lookups Vec length = 0
        data.extend_from_slice(&0_u32.to_le_bytes());

        data
    }

    #[test]
    fn decodes_valid_transaction_account() {
        let data = valid_transaction_data();

        let transaction = decode_transaction_account(&PROGRAM_ID, &data).unwrap();

        assert_eq!(
            transaction.consensus_account,
            Pubkey::new_from_array([1; 32])
        );

        assert_eq!(transaction.creator, Pubkey::new_from_array([2; 32]));

        assert_eq!(transaction.rent_collector, Pubkey::new_from_array([3; 32]));

        assert_eq!(transaction.index, 7);
        assert_eq!(transaction.payload.account_index, 1);

        assert_eq!(transaction.payload.ephemeral_signer_bumps, vec![9, 10]);

        let message = transaction.payload.message;

        assert_eq!(message.num_signers, 1);
        assert_eq!(message.num_writable_signers, 1);
        assert_eq!(message.num_writable_non_signers, 1);

        assert_eq!(message.account_keys.len(), 3);

        assert_eq!(message.account_keys[0], Pubkey::new_from_array([4; 32]));

        assert_eq!(message.account_keys[1], Pubkey::new_from_array([5; 32]));

        assert_eq!(message.account_keys[2], Pubkey::new_from_array([6; 32]));

        assert_eq!(message.instructions.len(), 1);

        let instruction = &message.instructions[0];

        assert_eq!(instruction.program_id_index, 2);
        assert_eq!(instruction.account_indexes, vec![0, 1]);
        assert_eq!(instruction.data, vec![10, 20, 30]);

        assert!(message.address_table_lookups.is_empty());
    }

    #[test]
    fn rejects_transaction_with_wrong_owner() {
        let data = valid_transaction_data();
        let wrong_owner = Pubkey::new_from_array([50; 32]);

        let result = decode_transaction_account(&wrong_owner, &data);

        assert!(matches!(result, Err(SdkError::WrongTransactionOwner)));
    }

    #[test]
    fn rejects_short_transaction_data() {
        let result = decode_transaction_account(&PROGRAM_ID, &[1, 2, 3]);

        assert!(matches!(result, Err(SdkError::TransactionDataTooShort)));
    }

    #[test]
    fn rejects_wrong_transaction_discriminator() {
        let mut data = valid_transaction_data();
        data[0] ^= 1;

        let result = decode_transaction_account(&PROGRAM_ID, &data);

        assert!(matches!(
            result,
            Err(SdkError::InvalidTransactionDiscriminator)
        ));
    }

    #[test]
    fn rejects_policy_payload() {
        let mut data = valid_transaction_data();

        // 8 byte discriminator
        // + three 32 byte Pubkeys
        // + one 8 byte u64
        let payload_variant_offset = 8 + 32 + 32 + 32 + 8;

        data[payload_variant_offset] = 1;

        let result = decode_transaction_account(&PROGRAM_ID, &data);

        assert!(matches!(result, Err(SdkError::UnsupportedPolicyPayload)));
    }

    #[test]
    fn rejects_unknown_payload_variant() {
        let mut data = valid_transaction_data();

        let payload_variant_offset = 8 + 32 + 32 + 32 + 8;

        data[payload_variant_offset] = 9;

        let result = decode_transaction_account(&PROGRAM_ID, &data);

        assert!(matches!(
            result,
            Err(SdkError::InvalidTransactionPayloadVariant(9))
        ));
    }

    #[test]
    fn rejects_incomplete_transaction_payload() {
        let mut data = valid_transaction_data();

        // Keep the fixed account fields and enum variant,
        // but remove the TransactionPayloadDetails body.
        let payload_variant_offset = 8 + 32 + 32 + 32 + 8;
        data.truncate(payload_variant_offset + 1);

        let result = decode_transaction_account(&PROGRAM_ID, &data);

        assert!(matches!(result, Err(SdkError::InvalidTransactionData(_))));
    }
}

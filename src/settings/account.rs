use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;

use crate::{
    SdkError, constant::PROGRAM_ID, helpers::anchor_discriminator, types::SmartAccountSigner,
};

#[derive(Debug, Clone, PartialEq, Eq, BorshDeserialize)]
pub struct SettingsAccount {
    pub seed: u128,
    pub settings_authority: Pubkey,
    pub threshold: u16,
    pub time_lock: u32,
    pub transaction_index: u64,
    pub stale_transaction_index: u64,
    pub archival_authority: Option<Pubkey>,
    pub archivable_after: u64,
    pub bump: u8,
    pub signers: Vec<SmartAccountSigner>,
    pub account_utilization: u8,
    pub policy_seed: Option<u64>,
    pub reserved2: u8,
}

pub fn decode_settings_account(owner: &Pubkey, data: &[u8]) -> Result<SettingsAccount, SdkError> {
    if owner != &PROGRAM_ID {
        return Err(SdkError::WrongSettingsOwner);
    }

    if data.len() < 8 {
        return Err(SdkError::SettingsDataTooShort);
    }

    let expected_discriminator = anchor_discriminator("account:Settings");

    if data[..8] != expected_discriminator {
        return Err(SdkError::InvalidSettingsDiscriminator);
    }

    let mut account_data = &data[8..];

    SettingsAccount::deserialize(&mut account_data).map_err(SdkError::InvalidSettingsData)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constant::PROGRAM_ID;

    fn valid_settings_data() -> Vec<u8> {
        let settings_authority = Pubkey::new_from_array([1; 32]);
        let first_signer = Pubkey::new_from_array([2; 32]);
        let second_signer = Pubkey::new_from_array([3; 32]);

        let mut data = anchor_discriminator("account:Settings").to_vec();

        data.extend_from_slice(&7_u128.to_le_bytes());
        data.extend_from_slice(settings_authority.as_ref());
        data.extend_from_slice(&2_u16.to_le_bytes());
        data.extend_from_slice(&300_u32.to_le_bytes());
        data.extend_from_slice(&10_u64.to_le_bytes());
        data.extend_from_slice(&4_u64.to_le_bytes());

        // archival_authority: None
        data.push(0);

        data.extend_from_slice(&1_000_u64.to_le_bytes());
        data.push(250);

        // Vec length: two signers
        data.extend_from_slice(&2_u32.to_le_bytes());

        data.extend_from_slice(first_signer.as_ref());
        data.push(7);

        data.extend_from_slice(second_signer.as_ref());
        data.push(3);

        data.push(2);

        // policy_seed: None
        data.push(0);

        data.push(0);

        // Extra allocated space for the two None option values.
        data.extend_from_slice(&[0; 40]);

        data
    }

    #[test]
    fn decodes_valid_settings_account() {
        let data = valid_settings_data();

        let settings = decode_settings_account(&PROGRAM_ID, &data).unwrap();

        assert_eq!(settings.seed, 7);
        assert_eq!(settings.settings_authority, Pubkey::new_from_array([1; 32]));
        assert_eq!(settings.threshold, 2);
        assert_eq!(settings.time_lock, 300);
        assert_eq!(settings.transaction_index, 10);
        assert_eq!(settings.stale_transaction_index, 4);
        assert_eq!(settings.archival_authority, None);
        assert_eq!(settings.archivable_after, 1_000);
        assert_eq!(settings.bump, 250);
        assert_eq!(settings.signers.len(), 2);
        assert_eq!(settings.signers[0].key, Pubkey::new_from_array([2; 32]));
        assert_eq!(settings.signers[0].permissions.mask, 7);
        assert_eq!(settings.signers[1].key, Pubkey::new_from_array([3; 32]));
        assert_eq!(settings.signers[1].permissions.mask, 3);
        assert_eq!(settings.account_utilization, 2);
        assert_eq!(settings.policy_seed, None);
        assert_eq!(settings.reserved2, 0);
    }

    #[test]
    fn rejects_settings_with_wrong_owner() {
        let data = valid_settings_data();
        let wrong_owner = Pubkey::new_from_array([9; 32]);

        let result = decode_settings_account(&wrong_owner, &data);

        assert!(matches!(result, Err(SdkError::WrongSettingsOwner)));
    }

    #[test]
    fn rejects_short_settings_data() {
        let result = decode_settings_account(&PROGRAM_ID, &[1, 2, 3]);

        assert!(matches!(result, Err(SdkError::SettingsDataTooShort)));
    }

    #[test]
    fn rejects_wrong_settings_discriminator() {
        let mut data = valid_settings_data();
        data[0] ^= 1;

        let result = decode_settings_account(&PROGRAM_ID, &data);

        assert!(matches!(
            result,
            Err(SdkError::InvalidSettingsDiscriminator)
        ));
    }

    #[test]
    fn rejects_invalid_settings_body() {
        let mut data = valid_settings_data();

        // Keep the discriminator but remove most of the account body.
        data.truncate(18);

        let result = decode_settings_account(&PROGRAM_ID, &data);

        assert!(matches!(result, Err(SdkError::InvalidSettingsData(_))));
    }
}

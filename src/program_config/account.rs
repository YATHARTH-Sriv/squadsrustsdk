use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;

use crate::{constant::PROGRAM_ID, error::SdkError, helpers::anchor_discriminator};

#[derive(Debug, Clone, BorshDeserialize)]
pub struct ProgramConfig {
    pub smart_account_index: u128,
    pub authority: Pubkey,
    pub smart_account_creation_fee: u64,
    pub treasury: Pubkey,
    pub reserved: [u8; 64],
}

pub fn decode_program_config(owner: &Pubkey, data: &[u8]) -> Result<ProgramConfig, SdkError> {
    if owner != &PROGRAM_ID {
        return Err(SdkError::WrongProgramConfigOwner);
    }

    if data.len() < 8 {
        return Err(SdkError::ProgramConfigDataTooShort);
    }

    let expected_discriminator = anchor_discriminator("account:ProgramConfig");

    if data[..8] != expected_discriminator {
        return Err(SdkError::InvalidProgramConfigDiscriminator);
    }

    ProgramConfig::try_from_slice(&data[8..]).map_err(SdkError::InvalidProgramConfigData)
}

#[cfg(test)]
mod tests {

    use solana_sdk::pubkey::Pubkey;

    use super::*;
    use crate::constant::PROGRAM_ID;

    fn valid_program_config_data() -> Vec<u8> {
        let authority = Pubkey::new_from_array([1; 32]);
        let treasury = Pubkey::new_from_array([2; 32]);

        let mut data = anchor_discriminator("account:ProgramConfig").to_vec();

        data.extend_from_slice(&7_u128.to_le_bytes());
        data.extend_from_slice(authority.as_ref());
        data.extend_from_slice(&5_000_u64.to_le_bytes());
        data.extend_from_slice(treasury.as_ref());
        data.extend_from_slice(&[0; 64]);

        data
    }

    #[test]
    fn decodes_valid_program_config() {
        let data = valid_program_config_data();

        let config = decode_program_config(&PROGRAM_ID, &data).unwrap();

        assert_eq!(config.smart_account_index, 7);

        assert_eq!(config.authority, Pubkey::new_from_array([1; 32]));

        assert_eq!(config.smart_account_creation_fee, 5_000);

        assert_eq!(config.treasury, Pubkey::new_from_array([2; 32]));
    }

    #[test]
    fn rejects_wrong_owner() {
        let data = valid_program_config_data();
        let wrong_owner = Pubkey::new_from_array([9; 32]);

        let result = decode_program_config(&wrong_owner, &data);

        assert!(matches!(result, Err(SdkError::WrongProgramConfigOwner)));
    }

    #[test]
    fn rejects_short_data() {
        let result = decode_program_config(&PROGRAM_ID, &[1, 2, 3]);

        assert!(matches!(result, Err(SdkError::ProgramConfigDataTooShort)));
    }

    #[test]
    fn rejects_wrong_discriminator() {
        let mut data = valid_program_config_data();
        data[0] ^= 1;

        let result = decode_program_config(&PROGRAM_ID, &data);

        assert!(matches!(
            result,
            Err(SdkError::InvalidProgramConfigDiscriminator)
        ));
    }
}

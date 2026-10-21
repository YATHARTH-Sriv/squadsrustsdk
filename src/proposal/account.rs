use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;

use crate::{SdkError, constant::PROGRAM_ID, helpers::anchor_discriminator};

#[derive(Debug, Clone, PartialEq, Eq, BorshDeserialize)]
pub enum ProposalStatus {
    Draft { timestamp: i64 },
    Active { timestamp: i64 },
    Rejected { timestamp: i64 },
    Approved { timestamp: i64 },
    Executing,
    Executed { timestamp: i64 },
    Cancelled { timestamp: i64 },
}

#[derive(Debug, Clone, PartialEq, Eq, BorshDeserialize)]
pub struct ProposalAccount {
    pub settings: Pubkey,
    pub transaction_index: u64,
    pub rent_collector: Pubkey,
    pub status: ProposalStatus,
    pub bump: u8,
    pub approved: Vec<Pubkey>,
    pub rejected: Vec<Pubkey>,
    pub cancelled: Vec<Pubkey>,
}

pub fn decode_proposal_account(owner: &Pubkey, data: &[u8]) -> Result<ProposalAccount, SdkError> {
    if owner != &PROGRAM_ID {
        return Err(SdkError::WrongProposalOwner);
    }

    if data.len() < 8 {
        return Err(SdkError::ProposalDataTooShort);
    }

    let expected_discriminator = anchor_discriminator("account:Proposal");

    if data[..8] != expected_discriminator {
        return Err(SdkError::InvalidProposalDiscriminator);
    }

    let mut account_data = &data[8..];

    ProposalAccount::deserialize(&mut account_data).map_err(SdkError::InvalidProposalData)
}

#[cfg(test)]
mod tests {
    use solana_sdk::pubkey::Pubkey;

    use super::{ProposalStatus, decode_proposal_account};

    use crate::{SdkError, constant::PROGRAM_ID, helpers::anchor_discriminator};

    const STATUS_OFFSET: usize = 8 + 32 + 8 + 32;

    fn valid_proposal_data() -> Vec<u8> {
        let mut data = anchor_discriminator("account:Proposal").to_vec();

        // settings
        data.extend_from_slice(&[1; 32]);

        // transaction_index
        data.extend_from_slice(&7_u64.to_le_bytes());

        // rent_collector
        data.extend_from_slice(&[2; 32]);

        // ProposalStatus::Approved is variant 3.
        data.push(3);

        // Approved timestamp
        data.extend_from_slice(&1_700_000_000_i64.to_le_bytes());

        // bump
        data.push(250);

        // approved: two public keys
        data.extend_from_slice(&2_u32.to_le_bytes());
        data.extend_from_slice(&[3; 32]);
        data.extend_from_slice(&[4; 32]);

        // rejected: one public key
        data.extend_from_slice(&1_u32.to_le_bytes());
        data.extend_from_slice(&[5; 32]);

        // cancelled: empty
        data.extend_from_slice(&0_u32.to_le_bytes());

        // Unused account allocation.
        data.extend_from_slice(&[0; 64]);

        data
    }

    #[test]
    fn decodes_valid_proposal_account() {
        let data = valid_proposal_data();

        let proposal = decode_proposal_account(&PROGRAM_ID, &data).unwrap();

        assert_eq!(proposal.settings, Pubkey::new_from_array([1; 32]));

        assert_eq!(proposal.transaction_index, 7);

        assert_eq!(proposal.rent_collector, Pubkey::new_from_array([2; 32]));

        assert_eq!(
            proposal.status,
            ProposalStatus::Approved {
                timestamp: 1_700_000_000,
            }
        );

        assert_eq!(proposal.bump, 250);

        assert_eq!(
            proposal.approved,
            vec![
                Pubkey::new_from_array([3; 32]),
                Pubkey::new_from_array([4; 32]),
            ]
        );

        assert_eq!(proposal.rejected, vec![Pubkey::new_from_array([5; 32])]);

        assert!(proposal.cancelled.is_empty());
    }

    #[test]
    fn decodes_executed_with_variant_five() {
        let mut data = valid_proposal_data();

        // Executed is variant 5 because deprecated Executing
        // still occupies variant 4.
        data[STATUS_OFFSET] = 5;

        let proposal = decode_proposal_account(&PROGRAM_ID, &data).unwrap();

        assert_eq!(
            proposal.status,
            ProposalStatus::Executed {
                timestamp: 1_700_000_000,
            }
        );
    }

    #[test]
    fn rejects_proposal_with_wrong_owner() {
        let data = valid_proposal_data();
        let wrong_owner = Pubkey::new_from_array([9; 32]);

        let result = decode_proposal_account(&wrong_owner, &data);

        assert!(matches!(result, Err(SdkError::WrongProposalOwner)));
    }

    #[test]
    fn rejects_short_proposal_data() {
        let result = decode_proposal_account(&PROGRAM_ID, &[1, 2, 3]);

        assert!(matches!(result, Err(SdkError::ProposalDataTooShort)));
    }

    #[test]
    fn rejects_wrong_proposal_discriminator() {
        let mut data = valid_proposal_data();
        data[0] ^= 1;

        let result = decode_proposal_account(&PROGRAM_ID, &data);

        assert!(matches!(
            result,
            Err(SdkError::InvalidProposalDiscriminator)
        ));
    }

    #[test]
    fn rejects_incomplete_proposal_body() {
        let mut data = valid_proposal_data();

        // Leave the discriminator and part of the settings key.
        data.truncate(20);

        let result = decode_proposal_account(&PROGRAM_ID, &data);

        assert!(matches!(result, Err(SdkError::InvalidProposalData(_))));
    }
}

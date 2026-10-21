use crate::constant::{
    PROGRAM_ID, SEED_PREFIX, SEED_PROGRAM_CONFIG, SEED_PROPOSAL, SEED_SETTINGS, SEED_SMART_ACCOUNT,
    SEED_TRANSACTION,
};
use solana_sdk::pubkey::Pubkey;

pub fn program_config_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[SEED_PREFIX, SEED_PROGRAM_CONFIG], &PROGRAM_ID)
}

pub fn settings_pda(settings_seed: u128) -> (Pubkey, u8) {
    let seed_bytes = settings_seed.to_le_bytes();
    Pubkey::find_program_address(
        &[SEED_PREFIX, SEED_SETTINGS, seed_bytes.as_ref()],
        &PROGRAM_ID,
    )
}

pub fn smart_account_pda(settings_pda: &Pubkey, account_index: u8) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            SEED_PREFIX,
            settings_pda.as_ref(),
            SEED_SMART_ACCOUNT,
            &[account_index],
        ],
        &PROGRAM_ID,
    )
}

pub fn proposal_pda(settings_pda: &Pubkey, transaction_index: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            SEED_PREFIX,
            settings_pda.as_ref(),
            SEED_TRANSACTION,
            &transaction_index.to_le_bytes(),
            SEED_PROPOSAL,
        ],
        &PROGRAM_ID,
    )
}

pub fn transaction_pda(settings_pda: &Pubkey, transaction_index: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            SEED_PREFIX,
            settings_pda.as_ref(),
            SEED_TRANSACTION,
            &transaction_index.to_le_bytes(),
        ],
        &PROGRAM_ID,
    )
}

#[cfg(test)]
mod tests {
    use solana_sdk::pubkey;

    use super::*;

    #[test]
    fn programconfigpda() {
        let result = program_config_pda();
        let expected_pubkey = pubkey!("GmY9kVi3FhrCUn2MJkzzpE6C5618YoHuGsgqHU78cKus");
        assert_eq!(result.0, expected_pubkey)
    }

    #[test]
    fn settingspda() {
        let result = settings_pda(2);
        println!("Address {:?}", result.0); // to see this in terminal we add cargo test -- --nocapture
        assert_eq!(
            result.0,
            pubkey!("9pits5NtSr7qjgYxf8CAiRbST3CfpsVs58ExNyW9YJnn")
        )
    }
}

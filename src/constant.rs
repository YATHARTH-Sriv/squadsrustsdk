use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;

pub const PROGRAM_ID: Pubkey = pubkey!("SMRTzfY6DfH5ik3TKiyLFfXexV8uSG3d2UksSCYdunG");
pub(crate) const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");

pub(crate) const SEED_PREFIX: &[u8] = b"smart_account";
pub(crate) const SEED_PROGRAM_CONFIG: &[u8] = b"program_config";
pub(crate) const SEED_SETTINGS: &[u8] = b"settings";
pub(crate) const SEED_SMART_ACCOUNT: &[u8] = b"smart_account";
pub(crate) const SEED_TRANSACTION: &[u8] = b"transaction";
pub(crate) const SEED_PROPOSAL: &[u8] = b"proposal";

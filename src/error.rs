use thiserror::Error;

#[derive(Debug, Error)]
pub enum SdkError {
    #[error("ProgramConfig account has an unexpected owner")]
    WrongProgramConfigOwner,

    #[error("ProgramConfig account data is too short")]
    ProgramConfigDataTooShort,

    #[error("ProgramConfig account has an invalid discriminator")]
    InvalidProgramConfigDiscriminator,

    #[error("failed to decode ProgramConfig account: {0}")]
    InvalidProgramConfigData(#[source] std::io::Error),

    // Settings Related Errors
    #[error("Settings account has an unexpected owner")]
    WrongSettingsOwner,

    #[error("Settings account data is too short")]
    SettingsDataTooShort,

    #[error("Settings account has an invalid discriminator")]
    InvalidSettingsDiscriminator,

    #[error("failed to decode Settings account: {0}")]
    InvalidSettingsData(#[source] std::io::Error),

    #[error("failed to serialize create smart account arguments: {0}")]
    CreateSmartAccountSerialization(#[source] std::io::Error),

    #[error("cannot derive the next Settings account because the smart account index overflowed")]
    SmartAccountIndexOverflow,

    #[error("failed to serialize authority Settings instruction arguments: {0}")]
    AuthoritySettingsSerialization(#[source] std::io::Error),

    // Transaction account errors
    #[error("Transaction account has an unexpected owner")]
    WrongTransactionOwner,

    #[error("Transaction account data is too short")]
    TransactionDataTooShort,

    #[error("Transaction account has an invalid discriminator")]
    InvalidTransactionDiscriminator,

    #[error("Policy transaction payloads are not supported in SDK V1")]
    UnsupportedPolicyPayload,

    #[error("Transaction account contains an unknown payload variant: {0}")]
    InvalidTransactionPayloadVariant(u8),

    #[error("failed to decode Transaction account: {0}")]
    InvalidTransactionData(#[source] std::io::Error),
    #[error("failed to compile smart account transaction message: {0}")]
    TransactionMessageCompilation(#[source] solana_sdk::message::CompileError),

    #[error("compiled transaction message contains too many writable non-signer accounts")]
    TransactionMessageAccountCountOverflow,

    #[error("{field} length {actual} exceeds the maximum supported length {maximum}")]
    TransactionMessageLengthOverflow {
        field: &'static str,
        actual: usize,
        maximum: usize,
    },

    #[error("failed to serialize create transaction arguments: {0}")]
    CreateTransactionSerialization(#[source] std::io::Error),

    #[error("cannot derive the next Transaction account because the transaction index overflowed")]
    TransactionIndexOverflow,

    // Proposal errors
    #[error("failed to serialize create proposal arguments: {0}")]
    CreateProposalSerialization(#[source] std::io::Error),

    #[error("Proposal account has an unexpected owner")]
    WrongProposalOwner,

    #[error("Proposal account data is too short")]
    ProposalDataTooShort,

    #[error("Proposal account has an invalid discriminator")]
    InvalidProposalDiscriminator,

    #[error("failed to decode Proposal account: {0}")]
    InvalidProposalData(#[source] std::io::Error),

    #[error("failed to serialize proposal vote arguments: {0}")]
    ProposalVoteSerialization(#[source] std::io::Error),

    #[error("address lookup tables are not supported for transaction execution in SDK V1")]
    UnsupportedExecutionAddressLookupTables,

    #[error("ephemeral signers are not supported for transaction execution in SDK V1")]
    UnsupportedExecutionEphemeralSigners,

    #[error("Transaction account contains an invalid message header")]
    InvalidTransactionMessageHeader,
}

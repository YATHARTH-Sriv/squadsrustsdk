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
    pda::proposal_pda,
    transaction::build_unsigned_transaction,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize)]
pub struct CreateProposalArgs {
    pub transaction_index: u64,
    pub draft: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreateProposalAccounts {
    pub consensus_account: Pubkey,
    pub proposal: Pubkey,
    pub creator: Pubkey,
    pub rent_payer: Pubkey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreateProposalInput {
    pub fee_payer: Pubkey,
    pub recent_blockhash: Hash,
    pub consensus_account: Pubkey,
    pub creator: Pubkey,
    pub rent_payer: Pubkey,
    pub transaction_index: u64,
    pub draft: bool,
}

#[derive(Debug)]
pub struct CreateProposalOutput {
    pub unsigned_transaction: Transaction,
    pub proposal_address: Pubkey,
    pub transaction_index: u64,
}

pub fn build_create_proposal_instruction(
    accounts: CreateProposalAccounts,
    args: CreateProposalArgs,
) -> Result<Instruction, SdkError> {
    let discriminator = anchor_discriminator("global:create_proposal");

    let serialized_args = borsh::to_vec(&args).map_err(SdkError::CreateProposalSerialization)?;

    let mut data = Vec::with_capacity(discriminator.len() + serialized_args.len());

    data.extend_from_slice(&discriminator);
    data.extend_from_slice(&serialized_args);

    Ok(Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(accounts.consensus_account, false),
            AccountMeta::new(accounts.proposal, false),
            AccountMeta::new_readonly(accounts.creator, true),
            AccountMeta::new(accounts.rent_payer, true),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
        ],
        data,
    })
}

pub fn build_create_proposal(input: CreateProposalInput) -> Result<CreateProposalOutput, SdkError> {
    let (proposal_address, _) = proposal_pda(&input.consensus_account, input.transaction_index);

    let instruction = build_create_proposal_instruction(
        CreateProposalAccounts {
            consensus_account: input.consensus_account,
            proposal: proposal_address,
            creator: input.creator,
            rent_payer: input.rent_payer,
        },
        CreateProposalArgs {
            transaction_index: input.transaction_index,
            draft: input.draft,
        },
    )?;

    let unsigned_transaction =
        build_unsigned_transaction(input.fee_payer, input.recent_blockhash, instruction);

    Ok(CreateProposalOutput {
        unsigned_transaction,
        proposal_address,
        transaction_index: input.transaction_index,
    })
}

#[cfg(test)]
mod tests {
    use solana_sdk::{hash::Hash, instruction::AccountMeta, pubkey::Pubkey, signature::Signature};

    use super::{
        CreateProposalAccounts, CreateProposalArgs, CreateProposalInput, build_create_proposal,
        build_create_proposal_instruction,
    };

    use crate::{
        constant::{PROGRAM_ID, SYSTEM_PROGRAM_ID},
        helpers::anchor_discriminator,
        pda::proposal_pda,
    };

    fn test_accounts() -> CreateProposalAccounts {
        CreateProposalAccounts {
            consensus_account: Pubkey::new_from_array([1; 32]),
            proposal: Pubkey::new_from_array([2; 32]),
            creator: Pubkey::new_from_array([3; 32]),
            rent_payer: Pubkey::new_from_array([4; 32]),
        }
    }

    #[test]
    fn builds_create_proposal_account_metadata() {
        let accounts = test_accounts();

        let instruction = build_create_proposal_instruction(
            accounts,
            CreateProposalArgs {
                transaction_index: 7,
                draft: false,
            },
        )
        .unwrap();

        assert_eq!(instruction.program_id, PROGRAM_ID);

        assert_eq!(
            instruction.accounts,
            vec![
                AccountMeta::new_readonly(accounts.consensus_account, false,),
                AccountMeta::new(accounts.proposal, false),
                AccountMeta::new_readonly(accounts.creator, true),
                AccountMeta::new(accounts.rent_payer, true),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false,),
                AccountMeta::new_readonly(PROGRAM_ID, false),
            ]
        );
    }

    #[test]
    fn serializes_create_proposal_instruction_data() {
        let instruction = build_create_proposal_instruction(
            test_accounts(),
            CreateProposalArgs {
                transaction_index: 7,
                draft: true,
            },
        )
        .unwrap();

        let mut expected_data = anchor_discriminator("global:create_proposal").to_vec();

        expected_data.extend_from_slice(&7_u64.to_le_bytes());

        // Borsh bool: false = 0, true = 1.
        expected_data.push(1);

        assert_eq!(instruction.data.len(), 17);
        assert_eq!(instruction.data, expected_data);
    }

    #[test]
    fn builds_complete_unsigned_create_proposal() {
        let fee_payer = Pubkey::new_from_array([10; 32]);
        let creator = Pubkey::new_from_array([11; 32]);
        let rent_payer = Pubkey::new_from_array([12; 32]);
        let consensus_account = Pubkey::new_from_array([13; 32]);
        let recent_blockhash = Hash::new_from_array([14; 32]);
        let transaction_index = 9;

        let output = build_create_proposal(CreateProposalInput {
            fee_payer,
            recent_blockhash,
            consensus_account,
            creator,
            rent_payer,
            transaction_index,
            draft: false,
        })
        .unwrap();

        let (expected_proposal_address, _) = proposal_pda(&consensus_account, transaction_index);

        assert_eq!(output.proposal_address, expected_proposal_address);

        assert_eq!(output.transaction_index, transaction_index);

        let message = &output.unsigned_transaction.message;

        assert_eq!(message.recent_blockhash, recent_blockhash);
        assert_eq!(message.account_keys[0], fee_payer);

        assert!(message.account_keys.contains(&consensus_account));
        assert!(message.account_keys.contains(&expected_proposal_address));
        assert!(message.account_keys.contains(&creator));
        assert!(message.account_keys.contains(&rent_payer));

        // The three addresses are different, so all three must sign.
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

        let instruction_program =
            message.account_keys[usize::from(create_instruction.program_id_index)];

        assert_eq!(instruction_program, PROGRAM_ID);

        assert_eq!(
            &create_instruction.data[..8],
            &anchor_discriminator("global:create_proposal")
        );

        assert_eq!(
            &create_instruction.data[8..16],
            &transaction_index.to_le_bytes()
        );

        assert_eq!(create_instruction.data[16], 0); // draft = false
    }
}

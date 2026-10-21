use borsh::BorshSerialize;
use solana_sdk::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    transaction::Transaction,
};

use crate::{
    SdkError, constant::PROGRAM_ID, helpers::anchor_discriminator, pda::proposal_pda,
    transaction::build_unsigned_transaction,
};

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize)]
pub struct VoteOnProposalArgs {
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoteOnProposalAccounts {
    pub consensus_account: Pubkey,
    pub signer: Pubkey,
    pub proposal: Pubkey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoteOnProposalInput {
    pub fee_payer: Pubkey,
    pub recent_blockhash: Hash,
    pub consensus_account: Pubkey,
    pub signer: Pubkey,
    pub transaction_index: u64,
    pub memo: Option<String>,
}

#[derive(Debug)]
pub struct VoteOnProposalOutput {
    pub unsigned_transaction: Transaction,
    pub proposal_address: Pubkey,
    pub transaction_index: u64,
}

enum ProposalVoteAction {
    Approve,
    Reject,
}

fn build_vote_on_proposal_instruction(
    instruction_name: &str,
    accounts: VoteOnProposalAccounts,
    args: &VoteOnProposalArgs,
) -> Result<Instruction, SdkError> {
    let discriminator = anchor_discriminator(instruction_name);

    let serialized_args = borsh::to_vec(args).map_err(SdkError::ProposalVoteSerialization)?;

    let mut data = Vec::with_capacity(discriminator.len() + serialized_args.len());

    data.extend_from_slice(&discriminator);
    data.extend_from_slice(&serialized_args);

    Ok(Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(accounts.consensus_account, false),
            AccountMeta::new(accounts.signer, true),
            AccountMeta::new(accounts.proposal, false),
            // Optional system_program is absent for approve/reject.
            AccountMeta::new_readonly(PROGRAM_ID, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
        ],
        data,
    })
}

fn build_vote_on_proposal(
    input: VoteOnProposalInput,
    action: ProposalVoteAction,
) -> Result<VoteOnProposalOutput, SdkError> {
    let (proposal_address, _) = proposal_pda(&input.consensus_account, input.transaction_index);

    let accounts = VoteOnProposalAccounts {
        consensus_account: input.consensus_account,
        signer: input.signer,
        proposal: proposal_address,
    };

    let args = VoteOnProposalArgs { memo: input.memo };

    let instruction = match action {
        ProposalVoteAction::Approve => build_approve_proposal_instruction(accounts, args)?,
        ProposalVoteAction::Reject => build_reject_proposal_instruction(accounts, args)?,
    };

    let unsigned_transaction =
        build_unsigned_transaction(input.fee_payer, input.recent_blockhash, instruction);

    Ok(VoteOnProposalOutput {
        unsigned_transaction,
        proposal_address,
        transaction_index: input.transaction_index,
    })
}

pub fn build_approve_proposal_instruction(
    accounts: VoteOnProposalAccounts,
    args: VoteOnProposalArgs,
) -> Result<Instruction, SdkError> {
    build_vote_on_proposal_instruction("global:approve_proposal", accounts, &args)
}

pub fn build_approve_proposal(
    input: VoteOnProposalInput,
) -> Result<VoteOnProposalOutput, SdkError> {
    build_vote_on_proposal(input, ProposalVoteAction::Approve)
}

pub fn build_reject_proposal_instruction(
    accounts: VoteOnProposalAccounts,
    args: VoteOnProposalArgs,
) -> Result<Instruction, SdkError> {
    build_vote_on_proposal_instruction("global:reject_proposal", accounts, &args)
}

pub fn build_reject_proposal(input: VoteOnProposalInput) -> Result<VoteOnProposalOutput, SdkError> {
    build_vote_on_proposal(input, ProposalVoteAction::Reject)
}

#[cfg(test)]
mod tests {
    use solana_sdk::{hash::Hash, instruction::AccountMeta, pubkey::Pubkey, signature::Signature};

    use super::{
        VoteOnProposalAccounts, VoteOnProposalArgs, VoteOnProposalInput, build_approve_proposal,
        build_approve_proposal_instruction, build_reject_proposal,
        build_reject_proposal_instruction,
    };

    use crate::{constant::PROGRAM_ID, helpers::anchor_discriminator, pda::proposal_pda};

    fn test_accounts() -> VoteOnProposalAccounts {
        VoteOnProposalAccounts {
            consensus_account: Pubkey::new_from_array([1; 32]),
            signer: Pubkey::new_from_array([2; 32]),
            proposal: Pubkey::new_from_array([3; 32]),
        }
    }

    fn test_vote_input() -> VoteOnProposalInput {
        VoteOnProposalInput {
            fee_payer: Pubkey::new_from_array([10; 32]),
            recent_blockhash: Hash::new_from_array([11; 32]),
            consensus_account: Pubkey::new_from_array([12; 32]),
            signer: Pubkey::new_from_array([13; 32]),
            transaction_index: 7,
            memo: Some("vote memo".to_string()),
        }
    }

    #[test]
    fn builds_vote_on_proposal_account_metadata() {
        let accounts = test_accounts();

        let instruction =
            build_approve_proposal_instruction(accounts, VoteOnProposalArgs { memo: None })
                .unwrap();

        assert_eq!(instruction.program_id, PROGRAM_ID);

        assert_eq!(
            instruction.accounts,
            vec![
                AccountMeta::new_readonly(accounts.consensus_account, false,),
                AccountMeta::new(accounts.signer, true),
                AccountMeta::new(accounts.proposal, false),
                AccountMeta::new_readonly(PROGRAM_ID, false),
                AccountMeta::new_readonly(PROGRAM_ID, false),
            ]
        );
    }

    #[test]
    fn serializes_distinct_vote_discriminators_and_shared_args() {
        let approve_instruction = build_approve_proposal_instruction(
            test_accounts(),
            VoteOnProposalArgs {
                memo: Some("hello".to_string()),
            },
        )
        .unwrap();

        let reject_instruction = build_reject_proposal_instruction(
            test_accounts(),
            VoteOnProposalArgs {
                memo: Some("hello".to_string()),
            },
        )
        .unwrap();

        assert_eq!(
            &approve_instruction.data[..8],
            &[136, 108, 102, 85, 98, 114, 7, 147]
        );

        assert_eq!(
            &reject_instruction.data[..8],
            &[114, 162, 164, 82, 191, 11, 102, 25]
        );

        let mut expected_args = vec![1];

        expected_args.extend_from_slice(&5_u32.to_le_bytes());

        expected_args.extend_from_slice(b"hello");

        assert_eq!(&approve_instruction.data[8..], expected_args.as_slice());

        assert_eq!(&reject_instruction.data[8..], expected_args.as_slice());
    }

    #[test]
    fn serializes_none_vote_memo() {
        let instruction =
            build_reject_proposal_instruction(test_accounts(), VoteOnProposalArgs { memo: None })
                .unwrap();

        // Eight-byte discriminator and one-byte Option::None.
        assert_eq!(instruction.data.len(), 9);
        assert_eq!(instruction.data[8], 0);
    }

    #[test]
    fn builds_complete_unsigned_approve_proposal() {
        let input = test_vote_input();

        let fee_payer = input.fee_payer;
        let recent_blockhash = input.recent_blockhash;
        let consensus_account = input.consensus_account;
        let signer = input.signer;
        let transaction_index = input.transaction_index;

        let output = build_approve_proposal(input).unwrap();

        let (expected_proposal, _) = proposal_pda(&consensus_account, transaction_index);

        assert_eq!(output.proposal_address, expected_proposal);
        assert_eq!(output.transaction_index, transaction_index);

        let transaction = &output.unsigned_transaction;
        let message = &transaction.message;

        assert_eq!(message.recent_blockhash, recent_blockhash);
        assert_eq!(message.account_keys[0], fee_payer);

        assert!(message.account_keys.contains(&consensus_account));
        assert!(message.account_keys.contains(&signer));
        assert!(message.account_keys.contains(&expected_proposal));

        // The fee payer and proposal signer are different.
        assert_eq!(message.header.num_required_signatures, 2);
        assert_eq!(transaction.signatures.len(), 2);

        assert!(
            transaction
                .signatures
                .iter()
                .all(|signature| signature == &Signature::default())
        );

        assert_eq!(message.instructions.len(), 1);

        let vote_instruction = &message.instructions[0];

        let instruction_program =
            message.account_keys[usize::from(vote_instruction.program_id_index)];

        assert_eq!(instruction_program, PROGRAM_ID);

        assert_eq!(
            &vote_instruction.data[..8],
            &anchor_discriminator("global:approve_proposal")
        );
    }

    #[test]
    fn builds_complete_unsigned_reject_proposal() {
        let input = test_vote_input();

        let consensus_account = input.consensus_account;
        let transaction_index = input.transaction_index;

        let output = build_reject_proposal(input).unwrap();

        let (expected_proposal, _) = proposal_pda(&consensus_account, transaction_index);

        assert_eq!(output.proposal_address, expected_proposal);
        assert_eq!(output.transaction_index, transaction_index);

        let message = &output.unsigned_transaction.message;

        assert_eq!(message.instructions.len(), 1);

        let vote_instruction = &message.instructions[0];

        let instruction_program =
            message.account_keys[usize::from(vote_instruction.program_id_index)];

        assert_eq!(instruction_program, PROGRAM_ID);

        assert_eq!(
            &vote_instruction.data[..8],
            &anchor_discriminator("global:reject_proposal")
        );
    }
}

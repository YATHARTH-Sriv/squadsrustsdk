mod account;
mod create;
mod vote;

pub use account::{ProposalAccount, ProposalStatus, decode_proposal_account};

pub use create::{
    CreateProposalAccounts, CreateProposalArgs, CreateProposalInput, CreateProposalOutput,
    build_create_proposal, build_create_proposal_instruction,
};

pub use vote::{
    VoteOnProposalAccounts, VoteOnProposalArgs, VoteOnProposalInput, VoteOnProposalOutput,
    build_approve_proposal, build_approve_proposal_instruction, build_reject_proposal,
    build_reject_proposal_instruction,
};

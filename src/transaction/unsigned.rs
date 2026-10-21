use solana_sdk::{
    hash::Hash, instruction::Instruction, message::Message, pubkey::Pubkey,
    transaction::Transaction,
};

pub(crate) fn build_unsigned_transaction(
    feepayer: Pubkey,
    recent_blockhash: Hash,
    instruction_to_be_included: Instruction,
) -> Transaction {
    let msg = Message::new_with_blockhash(
        &[instruction_to_be_included],
        Some(&feepayer),
        &recent_blockhash,
    );

    Transaction::new_unsigned(msg)
}

#[cfg(test)]
mod test {

    use solana_sdk::{hash::Hash, instruction::Instruction, pubkey::Pubkey, signature::Signature};

    use super::build_unsigned_transaction;

    #[test]
    fn builds_unsigned() {
        let fee_payer = Pubkey::new_from_array([1; 32]);
        let program_id = Pubkey::new_from_array([2; 32]);
        let recent_blockhash = Hash::new_from_array([3; 32]);

        let instruction = Instruction {
            program_id,
            accounts: vec![],
            data: vec![10, 20, 30],
        };

        let transaction = build_unsigned_transaction(fee_payer, recent_blockhash, instruction);

        assert_eq!(transaction.message.recent_blockhash, recent_blockhash);

        assert_eq!(transaction.message.account_keys[0], fee_payer);

        assert_eq!(transaction.signatures.len(), 1);

        assert_eq!(transaction.signatures[0], Signature::default());
    }
}

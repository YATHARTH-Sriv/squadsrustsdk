## Exploring The Sqauds Rust SDK :: Proposal 

## All Examples Are Using Surfpool as RPC

## Enviorment Variables ( Required For This )

```1. RPC_URL = http://127.0.0.1:8899```<br/>
```2. KEYPAIR_PATH = "  " ```
```3. SETTINGS_ADDRESS= ```
```4. TRANSACTION_INDEX= ```

---
### Proposal Creation , Approve and Reject For A Transaction 

``
Understanding Permissions 


( To be done )
``

```
1. build_create_proposal ( takes CreateProposalInput )  

    Create Proposal with argument CreateProposalInput 
    Returns CreateProposalOutput 
```
```
2. decode_proposal_account 

    decoding the Proposal Bytes into a Struct 
    Returns ProposalAccount 
```
```
3. build_approve_proposal ( takes VoteOnProposalInput )  

    Approve The vote 
    Returns VoteOnProposalOutput  
```
```
4. build_reject_proposal ( takes VoteOnProposalInput )  

    Reject The vote For that proposal 
    Returns VoteOnProposalOutputs  
```

---
---

### Running The Example 
---

``
We obviously need to have a Transaction ( we currently do no support Policy ) to Crate Proposal for right , so create a transaction and keep the txn index in env 
``


```
1. surfpool run
```
```
2. solana account SMRTzfY6DfH5ik3TKiyLFfXexV8uSG3d2UksSCYdunG
```
```
3. solana airdrop 100 [your-solana-address-used-for-sign-txns]
```
```
4. cargo run --examples proposal_create
```
```
5. cargo run --examples proposal_read
```
```
6. cargo run --examples proposal_approve
```
```
7. cargo run --examples proposal_reject
```

---
---
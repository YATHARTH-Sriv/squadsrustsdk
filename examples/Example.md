# Here are Some Examples To Use The SDK 

## Exploring The Sqauds Rust SDK :: program_config 

## Enviorment Variables ( Required For This )

```1. RPC_URL=http://127.0.0.1:8899```

---
### Reading the Program Config 

```
1. program_config_pda 

    Returns The program_config PDA
```
```
2. decode_program_config

    Returns Program Config Account from data bytes to struct
```
---
---

### Running The Example 
---

```
1. surfpool run
```
```
2. solana account SMRTzfY6DfH5ik3TKiyLFfXexV8uSG3d2UksSCYdunG
```
```
3. cargo run --examples programconfig_read
```

---
---

## Exploring The Sqauds Rust SDK :: Settings 

## Enviorment Variables ( Required For This )

```1. RPC_URL = http://127.0.0.1:8899```<br/>
```2. KEYPAIR_PATH = "  " ```

---
### Creating The Settings 

```
1. settings_pda ( takes setting_seed )  

    Settings Seed Comes From the smart_account_index from Program Config
    Returns The settings PDA
```
```
2. decode_settings_account

    Returns Settings Account from data bytes to struct
```
```
3. build_create_smart_account_transaction

    Arguments :
    - fee_payer: Pubkey,
    - recent_blockhash: Hash,
    - program_config: &ProgramConfig,
    - creator: Pubkey,
    - args: CreateSmartAccountArgs,
```
```
4. build_add_signer_as_authority_transaction

    Arguments :
    - fee_payer: Pubkey,
    - recent_blockhash: Hash,
    - accounts: AuthoritySettingsAccounts,
    - args: AddSignerArgs,
```
```
5. build_remove_signer_as_authority_transaction

    Arguments :
    - fee_payer: Pubkey,
    - recent_blockhash: Hash,
    - accounts: AuthoritySettingsAccounts,
    - args: RemoveSignerArgs,
```
```
6. build_change_threshold_as_authority_transaction

    Arguments :
    - fee_payer: Pubkey,
    - recent_blockhash: Hash,
    - accounts: AuthoritySettingsAccounts,
    - args: ChangeThresholdArgs,
```
```
7. build_set_time_lock_as_authority_transaction

    Arguments :
    - fee_payer: Pubkey,
    - recent_blockhash: Hash,
    - accounts: AuthoritySettingsAccounts,
    - args: SetTimeLockArgs,
```

---
---

### Running The Example 
---

``
One Consideration To take is create a settings account and save it to env so that you can keep testing the further examples as the other than create example i tried to kept the relevant functions only in thier respective files
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
4. cargo run --examples settings_create
```
```
5. cargo run --examples settings_read
```
```
6. cargo run --examples settings_adding_signer
```
```
7. cargo run --examples settings_change_threshold
```
```
8. cargo run --examples settings_remove_signer
```
```
9. cargo run --examples settings_set_timelock
```

---
---

## Exploring The Sqauds Rust SDK :: Proposal 

## Enviorment Variables ( Required For This )

```1. RPC_URL = http://127.0.0.1:8899```<br/>
```2. KEYPAIR_PATH = "  " ```
```3. SETTINGS_ADDRESS= ```
```4. TRANSACTION_INDEX= ```

---
### Proposal Creation , Approve and Reject For A Transaction 


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


## Exploring The Sqauds Rust SDK :: Transaction 

## Enviorment Variables ( Required For This )

```1. RPC_URL = http://127.0.0.1:8899```<br/>
```2. KEYPAIR_PATH = "  " ```
```3. SETTINGS_ADDRESS= ```
```4. TRANSACTION_INDEX= ```

---

```
1. build_create_transaction ( takes CreateTransactionInput )  

    Create Txn with argument CreateTransactionInput 
    Returns CreateTransactionOutput 
```
```
2. decode_transaction_account (  )  

    decodes the data bytes and return the struct
```
```
3. build_execute_transaction ( takes ExecuteTransactionInput )  

    Execute transaction 
```


---
---

### Running The Example 
---

``
Store the txn index and also fund the smart account to do txn 
``

```
    cargo run --example transaction_create
```
```
1. solana airdrop 100 [smart account]
```
```
After proposal is approved you can execute
```
```
2. cargo run --example transaction_read
```
```
3. cargo run --example transaction_execute
```


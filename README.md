# Rust SDK squadsrust_sdk 

A focused Rust SDK for interacting with the Squads Smart Account Program 

- Derives Squads PDAs and decodes supported onchain accounts.
- Builds unsigned Solana transactions for supported Settings, Proposal, and Transaction operations.
- RPC and signing the transactions is handled by user we do not need any of your secrets
- Does not depend on `coral-xyz/anchor` or require an IDL. 

## Installation 

    cargo add squadsrust_sdk

## Build on Squads Smart Account Program v0.1 

`` Things We have Covered ``<br/>

### PDA Derivations 
    - program config 
    - settings 
    - smart account
    - proposal 
    - transaction 

### Settings  
    - create 
    - decode ( Basically Convert From bytes To ) 
    - Add Signer
    - Change Threshold 
    - Remove Signer
    - Set Timelock

### Proposal  
    - create 
    - decode 
    - approve
    - reject 

### Transaction  
    - create 
    - decode 
    - approve
    - reject 

### Policies 
    - ( To Be Continued Before Gta VI ) 

---

### PDA DERIVATIONS :

### With squadsrust_sdk :: pda :: { } , from the sdk get all relevant PDA's needed 

``` All PDA's which can be derived with the current SDK ```

`` 1. Program Config ``
```
squadsrust_sdk::pda::program_config_pda (  )
```
`` 2. Settings  ``<br/><br/>
`` Arguments : Settings_seed comes from program_config smart_account_index ``<br/>

```
squadsrust_sdk::pda::settings_pda ( settings_seed )    
```

`` 3. Smart Account PDA ``<br/><br/>
`` Arguments : Settings PDA and Account Index as well    ``
```
squadsrust_sdk::pda::smart_account_pda ( settings_pda , account_index )
```

`` 4. Proposal PDA ``<br/><br/>
`` Arguments : Settings PDA and Transaction index``
```
squadsrust_sdk::pda::proposal_pda ( settings_pda , transaction_index )
```
`` 5. Transaction PDA ``<br/><br/>
`` Arguments : Settings PDA and Transaction index``
```
squadsrust_sdk::pda::transaction_pda ( settings_pda , transaction_index ) 
```
---
---
### Program Config :

### With squadsrust_sdk :: program_config

#### Decode Program Config 
    - use squadsrust_sdk::{program_config::decode_program_config}

``` As per the onchain Contract As users you can't do anything substantial related to Program Config other than decoding the struct for :```

- creation fee ( well thier is none as of now )
- smart_account_index to create the next settings account 
---
---

## Settings :

### With squadsrust_sdk :: settings :: { }

### 1. Decode Settings Account 

    - use squadsrust_sdk::{settings::decode_settings_account} ,

- Basically Decode the Settings Account 
---

### 2. Building The Settings Account Unsigned Transaction 

    - use squadsrust_sdk::{settings::build_create_smart_account_transaction} 

- Returns CreateSmartAccountTransaction 
- fields :  transaction,  settings,  settings_seed
---
<br/>

### 3. Building The Add Signer To Settings Account Unsigned Transaction 

    - use squadsrust_sdk::{settings::build_add_signer_as_authority_transaction} ```

- Returns Unsigned Transaction 
---
<br/>

### 4. Building The Change Threshold To Settings Account Unsigned Transaction 

    - use squadsrust_sdk::{settings::build_change_threshold_as_authority_transaction} 

- Returns Unsigned Transaction 
---
<br/>

### 5. Building The Remove Signer To Settings Account Unsigned Transaction 

    - use squadsrust_sdk::{settings::build_remove_signer_as_authority_transaction} 

- Returns Unsigned Transaction 
---
<br/>

### 6. Building The Set TimeLock To Settings Account Unsigned Transaction 

    - use squadsrust_sdk::{settings::build_set_time_lock_as_authority_transaction} 

- Returns Unsigned Transaction 
---
<br/>

---
---

## Proposal 

### With squadsrust_sdk :: proposal :: { }

### 1. Create Proposal Account 

    -  use squadsrust_sdk::{proposal::build_create_proposal} 

- Basically Creates the Proposal Account 
- Arguments is CreateProposalInput struct 
---

### 2. Decode Proposal Account 

    - use squadsrust_sdk::{proposal::decode_proposal_account} 

- Basically Decode the Proposal Account 
---

### 3. Approve Vote To Proposal Account 

    - use squadsrust_sdk::{proposal::build_approve_proposal} ,

- Approve The Vote Towards The Transaction Proposal
- VoteOnProposalInput
---

### 4. Reject Vote To Proposal Account 

    - use squadsrust_sdk::{proposal::build_reject_proposal} ,

- Reject The Vote Towards The Transaction Proposal
- VoteOnProposalInput
---
<br/>

---
---

### Transaction  

### With squadsrust_sdk :: transaction :: { }


### 1. Decode Transaction Account
    - use squadsrust_sdk::transaction::decode_transaction_account;
- Decodes the onchain Transaction account.
- Returns TransactionAccount.
---
<br/>

### 2. Build The Create Transaction Unsigned Transaction
   use squadsrust_sdk::transaction::{build_create_transaction, CreateTransactionInput};
- Creates an unsigned transaction that stores one or more inner Solana instructions in a new Transaction account.
- Returns CreateTransactionOutput.

---
<br/>

### 3. Build The Execute Transaction Unsigned Transaction
    
    use squadsrust_sdk::transaction::{build_execute_transaction, ExecuteTransactionInput};

- Builds an unsigned transaction that executes the instructions stored inside a Transaction account.
- The associated Proposal must be approved and its time lock must have passed.
- Returns ExecuteTransactionOutput.

---
<br/>

``` Some More Transaction Related Stuff ```
<br/>

### 4. Compile Inner Instructions Into A Smart Account Message
    use squadsrust_sdk::transaction::compile_transaction_message;

- Converts normal Solana Instruction values into SmartAccountTransactionMessage.
- Collects and orders the account keys.
- Records signer and writable account information.
- Converts each instruction into SmartAccountCompiledInstruction.
---
<br/>

### 5. Serialize A Smart Account Transaction Message
    
     use squadsrust_sdk::transaction::serialize_transaction_message;


- Serializes SmartAccountTransactionMessage into bytes using Borsh.
- These bytes are stored inside the onchain Transaction account.
- This is handled internally by build_create_transaction for normal SDK usage.

---

<br/>

### Mostly Users need to think about 

    - build_create_transaction

    - build_execute_transaction

    -decode_transaction_account

----

## Going Ahead
( Need To Work On Supporting Policies and Also Create Examples and Tests for more than just SOL transfers )
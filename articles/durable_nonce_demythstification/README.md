# Drift Exploit & Durable Nonce Demystification

The multitude of technical details surrounding the Drift Protocol incident on April 1, 2026, acts as a barrier to forming an objective picture of the events for both observers and the affected users. 

Unfortunately, this leads to a situation where knowledge gaps are filled not with objective data, but with PR noise and biased perspectives favorable to a specific circle of interested parties. 

One such aspect is the `Durable Nonce` and its role in the Drift incident. 

In this material, we demystify what has been hidden from the users' eyes and present the hard facts.

## Understanding Durable Nonces

A Durable Nonce is a specialized account type in Solana that stores a current nonce value (with specific 32 byte hash). This hash can be used by an authorized address (nonce Authority - also stored within the account) as a hash for signing a transaction.

A transaction signed with such a hash can be successfully executed as long as the nonce account stores that exact value. A critical point - the first instruction in such a transaction must be a specific command to the System Program that ensures the update of the nonce value used for signing ([advance_nonce_account](https://github.com/anza-xyz/agave/blob/v3.1.8/programs/system/src/system_instruction.rs#L22)).

The moment the blockchain receives and executes such a transaction, the nonce account is updated with a new hash for future use.

This design enables offline transaction signing without using the standard `recent_blockhash` method, which limits the transaction's validity window to 150 blocks (approx. 90 seconds).

> Let's use an analogy from real life: 
>
> Imagine that a standard Solana transaction is a bank check that is only valid for 90 seconds. If you don't make it to the teller in time, the check simply becomes invalid.
>
> A Durable Nonce is a `post-dated check`. You sign it today, but you leave the date blank. This check can sit in anyone's pocket, but it can only be cashed if it bears `your signature`. 

## Creating and Using a Durable Nonce Account

Absolutely anyone willing to pay a small standard rent fee can create a `Durable Nonce` account on Solana blockchain. 

However, only its `Authority` can use an account nonce value, as only the Authority has the power to execute the `advance_nonce_account` instruction and must serve as its signer.

As proof, you can refer to the official [Solana documentation](https://solana.com/docs/core/transactions/durable-nonces) or explore the system_program [source code](https://github.com/anza-xyz/agave/blob/v3.1.8/programs/system/src/system_instruction.rs#L22). 

The creator of the Durable Nonce account assigns the Authority, or the Authority themselves can transfer control later (via [system_program::authorize_nonce_account](https://github.com/anza-xyz/agave/blob/v3.1.8/programs/system/src/system_instruction.rs#L206)). 

Simply put: if you want to use a nonce account (e.g., for offline signing), you don't have to create it yourself - a third party can create it for you and designate you as the Authority. Conversely, you can create such an account for anyone else.

> The Bank Check Analogy:
>
> If you see a nonce account on the network where you are designated as the `Authority`, it means someone has prepared a blank "long-lived" form that `only you` can sign.

Thus, four durable nonce accounts ([1](https://solscan.io/tx/43GonZECYsSAFVXUJvTuY564MX4YG8CK2Bx8v8zAkQnPs15XmiJ9t9xrucHscefRyhASHvZpWZ25zCPXYhQiesk7), [2](https://solscan.io/tx/LJuBqSWpfW6GSgWi2v64B6czZfd618ZXxPLBTQt2tSgF4hHCUgfYvUuAMxeSmfL4FnS8Wt9cKNaSK9YNke7kTz1), [3](https://solscan.io/tx/2S2WqzzbbQaPnHHuG6zVSSLp7jSn8wUDYLkKvqA8XLUT3UfUVCiUWN4ukwNz1zCDTnBqYNv3ERaNDPMQZ35ZCxjJ), [4](https://solscan.io/tx/2emng4wK82hAhaBHUCGFckEyvUcXVf7yWv8dTotvEgYqVcRiDStEUooTXzrpZyS1du5PfPKcZyTjfpdac5pyXGHw)) were created by the [same initiator](https://solscan.io/account/FMJnBkVpHj5JzN7w4XFysCwY931CYSYk1DsXzqNi7YPF) at approximately the same time, one of which designated an active member of the Drift security council as its Authority. Seven days later, an additional account [was created](https://solscan.io/tx/59yWWZjnLeu3WP6Dqj4NW21NWHhdNwkToCbypdNrAHKmhk5C37ZDUygbuDVPSN2XqYzME88k6Ss3sBKGdrmrWrX3) by the [same source](https://solscan.io/account/FMJnBkVpHj5JzN7w4XFysCwY931CYSYk1DsXzqNi7YPF), naming yet another council member as the Authority.

## How Durable Nonces Were Used During the Exploit

As discussed in previous [article](https://github.com/DriftVictimsCommittee/drift-forensics/tree/main/articles/exploit-sovereign-view), the protocol takeover occurred by replacing the administrator in the state of the `Drift V2 core program`. Two security council accounts were involved, executing two governance transactions ([Transaction A](https://solscan.io/tx/2HvMSgDEfKhNryYZKhjowrBY55rUx5MWtcWkG9hqxZCFBaTiahPwfynP1dxBSRk9s5UTVc8LFeS4Btvkm9pc2C4H) and [Transaction B](https://solscan.io/tx/4BKBmAJn6TdsENij7CsVbyMVLJU1tX27nfrMM1zgKv1bs2KJy6Am2NqdA3nJm4g9C6eC64UAf5sNs974ygB9RsN1)):

- account [39JyWrdbVdRqjzw9yyEjxNtTbTKcTPLdtdCgbz7C7Aq8](https://solscan.io/account/39JyWrdbVdRqjzw9yyEjxNtTbTKcTPLdtdCgbz7C7Aq8) - utilizing [7s7s6saC5LHZoLyBXLM3pCjpWaA7meyQdP8NiH9ktAeC](https://solscan.io/account/7s7s6saC5LHZoLyBXLM3pCjpWaA7meyQdP8NiH9ktAeC) durable nonce initiated a proposal to change the Drift V2 program `admin` and approved it as a council member (`Transaction A`).

- account [6UJbu9ut5VAsFYQFgPEa5xPfoyF5bB5oi4EknFPvu924](https://solscan.io/account/6UJbu9ut5VAsFYQFgPEa5xPfoyF5bB5oi4EknFPvu924) - utilizing [EmYEryTDXtuVCxrjNqJXbiwr4hfiJajd4g5P58vvhQnc](https://solscan.io/account/EmYEryTDXtuVCxrjNqJXbiwr4hfiJajd4g5P58vvhQnc) durable nonce approved the proposal and, having sufficient quorum (2/5), executed the transaction changing the `admin` to account `H7PiGqqUaanBovwKgEtreJbKmQe6dbq6VTrw6guy7ZgL` (`Transaction B`).

## A Detailed Look at the Involved Council Members

As we demonstrated [here](https://github.com/DriftVictimsCommittee/drift-forensics/tree/main/articles/exploit-sovereign-view), before the migration, the multisig configuration included 5 members. One of them was account `39Jy...7Aq8`, which was subsequently moved to the security council of the new multisig. This account initiated `Transaction A` on the day of the exploit.

During the "planned" multisig migration on March 25, 2026, a new multisig [was created](https://solscan.io/tx/3aKRMvoc6JRoZCMrefvioMdZMrCJXJ1hFj3ENvGHhDbJW2AFGWBkLZpWp1A2twbLoLjWivRQACWKbJEJRF8xoQrr). Along with 4 new members, it included this exact address (`39Jy...7Aq8`) as the retained participant. 

Meanwhile, one of the 4 new members was the account (`6UJ...924`) that executed `Transaction B`.

Thus, if we consider that the exploit fully depended on compromising 2 out of 5 council accounts, and both were already compromised at the time of migration, then Drift effectively created a scenario where the new council immediately met the minimum quorum (2/5) for a malicious takeover.

> Now, imagine a bank that, knowing a manager's checkbook has been stolen, not only fails to revoke their signing authority but also appoints them as the head of a new branch.

## What Drift Failed to Do Before Migration

Over `39 hours` (!) passed between the creation of the nonce account for the `39Jy...7Aq8` council member and the multisig migration, as a result of which `39Jy...7Aq8` was retained as a council participant:
* Nonce account for `39Jy...7Aq8` [creation](https://solscan.io/tx/LJuBqSWpfW6GSgWi2v64B6czZfd618ZXxPLBTQt2tSgF4hHCUgfYvUuAMxeSmfL4FnS8Wt9cKNaSK9YNke7kTz1): `01:22:06 Mar 24, 2026 (UTC)`
* New multisig with `39Jy...7Aq8` council member [creation](https://solscan.io/tx/3aKRMvoc6JRoZCMrefvioMdZMrCJXJ1hFj3ENvGHhDbJW2AFGWBkLZpWp1A2twbLoLjWivRQACWKbJEJRF8xoQrr): `16:58:31 Mar 25, 2026 (UTC)`

What should a protocol managing hundreds of millions of dollars do during a planned action involving critical security?

When creating a new security council, one must check for any `markers` indicating compromise. Nonce accounts are good markers and a check would involve `querying` the network for `existing nonce accounts` where the Authority matches the public key of any new council member.

In case the team did not initiate found accounts, it should have been a `massive red flag`, triggering an immediate investigation and the suspension of planned actions. 

The fact that Drift `proceeded` despite these accounts existing points to two possibilities:
1.  The team failed to perform `basic` due diligence.
2.  The team `knew` and proceeded anyway.

Below is the very simple code that could have detected such red flag:

```rust
// ...

fn durable_nonce_warns(
    rpc_client: &RpcClient,
    council_members: &[Pubkey],
    whitelisted_nonce_accounts: &[Pubkey]
) -> Result<Vec<(Pubkey, Pubkey)>, Box<dyn std::error::Error>> {
    let mut warn_nonce_accounts = Vec::new();

    for council_member in council_members {
        let filters = vec![
            // 1. Datasize filter (80 bytes total)
            RpcFilterType::DataSize(80),
            // 2. State == 1 (Initialized)
            RpcFilterType::Memcmp(Memcmp::new_raw_bytes(0, vec![1, 0, 0, 0])),
            // 3. Authority pubkey filter (offset = 8)
            RpcFilterType::Memcmp(Memcmp::new_base58_encoded(8, &council_member.to_bytes())),
        ];

        let config = RpcProgramAccountsConfig {
            filters: Some(filters),
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                data_slice: Some(UiDataSliceConfig {
                    offset: 0,
                    length: 40,
                }),
                commitment: Some(CommitmentConfig::confirmed()),
                min_context_slot: None,
            },
            with_context: Some(false),
            sort_results: Some(true),
        };

        // Fetch with config
        let accounts = rpc_client.get_program_ui_accounts_with_config(
            &SYSTEM_PROGRAM_ID,
            config,
        )?;

        // Loop and compare with whitelist
        for (nonce_pubkey, _nonce_account) in accounts {
            if !whitelisted_nonce_accounts.contains(&nonce_pubkey) {
                warn_nonce_accounts.push((nonce_pubkey, *council_member));
            }
        }
    }

    Ok(warn_nonce_accounts)
}
```

This is merely the simplest way to detect potential issues. However, a company spending thousands of dollars on security should surely have far more systematic methods in its arsenal.

## Claims Regarding "Pre-signed" Transactions

It is impossible to prove, and therefore incorrect to claim, that any transaction using a `durable nonce` was signed at a `specific moment` in time. This information does not exist on-chain. We simply know they were signed using nonces, but the exact timing remains `unverifiable`.

We are seeing widespread speculation that an attacker used `social engineering` to get two transactions signed at different times. This is nothing more than PR noise. 

Transactions using durable nonces could have been signed `immediately before` execution, looking identical to transactions signed days prior. 

This is especially true if we consider the possibility of a case, where durable nonces were used simply as a mask.

## Conclusion

In addition to the existing perspectives on the incident, the following conclusions can be drawn:

- **Failure to Act**: While performing the multisig migration, Drift could have easily detected the presence of these durable nonce accounts and responded appropriately by excluding the presumably compromised participant, increasing the signature threshold, or implementing a timelock.

- **A Visible Trail**: The very existence of durable nonce accounts on-chain, with Drift council members set as the nonce authority, could have exposed the plans of those intending to exploit them and undermined their intentions, provided the team was actually monitoring the network.

- **Redundancy of Nonces**: If the exploiters had indeed gained control over the private keys of two council members, both transactions changing the program admin could have been executed without using durable nonces at all.

- **Calculated Masking**: In a scenario where durable nonces were not technically required, their intentional use suggests a deliberate attempt to project a specific behavioral pattern (a "smokescreen").

- **The Forensic Dead-end**: There is no objective, on-chain verifiable evidence to prove exactly when `Transactions A` and `Transactions B` which utilized the hashes from the two durable nonce accounts were actually signed.

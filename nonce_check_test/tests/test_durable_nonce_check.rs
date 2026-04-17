use nonce_check_test::check_durable_nonces;
use solana_client::{rpc_client::RpcClient, rpc_config::CommitmentConfig};
use solana_pubkey::Pubkey;

#[test]
fn durable_nonce_check() {
    // Connect to local test validator running the pre-loaded exploit state
    let rpc_client = RpcClient::new_with_commitment(
        "http://127.0.0.1:8899".to_string(),
        CommitmentConfig::confirmed(),
    );

    // Security Council members of the new Multisig (post-migration)
    // Includes the compromised accounts involved in the Drift exploit
    let council_members = vec![
        Pubkey::from_str_const("39JyWrdbVdRqjzw9yyEjxNtTbTKcTPLdtdCgbz7C7Aq8"),
        Pubkey::from_str_const("6UJbu9ut5VAsFYQFgPEa5xPfoyF5bB5oi4EknFPvu924"),
        Pubkey::from_str_const("7TxYEAKSHRuCs1QpxssoeuaewqdQzHf93EKQP7bNYYxh"),
        Pubkey::from_str_const("13GXtbGV8mNfNLDNbVKPrTcHTpfZ4CYrXviRCZmyxQvj"),
        Pubkey::from_str_const("HgjySRE1j9T2NwFrGbK2hXk4Przoz31xdciedmt6CHF6"),
    ];

    // Whitelist of known legitimate nonce accounts.
    // In a real-world audit, this would contain protocol-owned nonces.
    // For this PoC, we assume no legitimate nonces exist, so any found is suspicious.
    let whitelisted_nonce_accounts: Vec<Pubkey> = vec![];

    println!("Scanning for durable nonces controlled by council members...");
    
    let warn_accounts =
        check_durable_nonces(&rpc_client, council_members, whitelisted_nonce_accounts)
            .expect("RPC call failed");

    println!("Found {} suspicious nonce account(s):", warn_accounts.len());
    for (nonce_pk, authority_pk) in &warn_accounts {
        println!("  Nonce: {:?} controlled by: {:?}", nonce_pk, authority_pk);
    }
      
    // The test passes if we detect at least one unauthorized nonce account
    assert!(!warn_accounts.is_empty(), "Failed to detect suspicious durable nonce accounts");
}

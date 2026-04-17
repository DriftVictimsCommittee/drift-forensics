use solana_account_decoder_client_types::{UiAccountEncoding, UiDataSliceConfig};
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{CommitmentConfig, RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, RpcFilterType},
};
use solana_pubkey::Pubkey;
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;

/// Checks for unauthorized Durable Nonce accounts associated with specific council members.
///
/// This function queries the System Program for initialized nonce accounts (80 bytes)
/// where the authority matches any of the provided `council_members`.
/// It returns a list of (nonce_account_pubkey, council_member_pubkey) pairs that are NOT
/// present in the `whitelisted_nonce_accounts` list.
///
/// # Arguments
/// * `rpc_client` - An active RPC client connected to a Solana cluster.
/// * `council_members` - List of public keys to check as potential nonce authorities.
/// * `whitelisted_nonce_accounts` - List of known legitimate nonce account addresses to exclude.
///
/// # Returns
/// A vector of tuples containing the suspicious nonce account pubkey and the associated council member pubkey.
pub fn check_durable_nonces(
    rpc_client: &RpcClient,
    council_members: Vec<Pubkey>,
    whitelisted_nonce_accounts: Vec<Pubkey>,
) -> Result<Vec<(Pubkey, Pubkey)>, Box<dyn std::error::Error>> {
    let mut warn_nonce_accounts = Vec::new();

    for council_member in council_members {
        // Filters to identify valid, initialized Durable Nonce accounts owned by the council member:
        // 1. DataSize(80): Standard size of a NonceAccount.
        // 2. Memcmp(0, [1,0,0,0]): Discriminator/State == Initialized (u32 little-endian).
        // 3. Memcmp(8, <pubkey>): Authority field offset at byte 8.
        let filters = vec![
            // 1. Datasize filter
            RpcFilterType::DataSize(80),
            // 2. Discriminator filter == 1 (Initialized)
            RpcFilterType::Memcmp(Memcmp::new_raw_bytes(0, vec![1, 0, 0, 0])),
            // 3. Authority pubkey filter
            RpcFilterType::Memcmp(Memcmp::new_base58_encoded(8, &council_member.to_bytes())),
        ];

        let config = RpcProgramAccountsConfig {
            filters: Some(filters),
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                // Fetch only the first 40 bytes (Discriminator + State + Authority) to minimize bandwidth.
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

        // Query the System Program for matching accounts.
        let accounts =
            rpc_client.get_program_ui_accounts_with_config(&SYSTEM_PROGRAM_ID, config)?;

        // Filter out whitelisted accounts and collect warnings.
        for (nonce_pubkey, _nonce_account) in accounts {
            if !whitelisted_nonce_accounts.contains(&nonce_pubkey) {
                warn_nonce_accounts.push((nonce_pubkey, council_member));
            }
        }
    }

    Ok(warn_nonce_accounts)
}

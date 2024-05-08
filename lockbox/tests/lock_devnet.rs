use std::str::FromStr;

use conjunto_lockbox::{
    accounts::{RpcAccountProvider, RpcAccountProviderConfig},
    AccountLockState, AccountLockStateProvider,
};
use solana_sdk::{pubkey::Pubkey, system_program};

#[tokio::test]
async fn test_known_delegation() {
    // NOTE: this test depends on these accounts being present on devnet
    // and properly locked

    let delegated_addr = "8k2V7EzQtNg38Gi9HK5ZtQYp1YpGKNGrMcuGa737gZX4";
    let delegated_id = Pubkey::from_str(delegated_addr).unwrap();

    let buffer_addr = "E8NdkAGLLC3qnvphsXhqkjkXpRkdoiDpicSTTQJySVtG";
    let buffer_id = Pubkey::from_str(buffer_addr).unwrap();

    let delegation_addr = "CkieZJmrj6dLhwteG69LSutpWwRHiDJY9S8ua7xJ7CRW";
    let delegation_id = Pubkey::from_str(delegation_addr).unwrap();

    let lockstate_provider =
        AccountLockStateProvider::<RpcAccountProvider>::new(
            RpcAccountProviderConfig::default(),
        );

    let state = lockstate_provider
        .try_lockstate_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert_eq!(
        state,
        AccountLockState::Locked {
            delegated_id,
            buffer_pda: buffer_id,
            delegation_pda: delegation_id
        }
    );
}

#[tokio::test]
async fn test_system_account_not_delegated() {
    let delegated_id = system_program::id();

    let lockstate_provider =
        AccountLockStateProvider::<RpcAccountProvider>::new(
            RpcAccountProviderConfig::default(),
        );

    let state = lockstate_provider
        .try_lockstate_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert_eq!(state, AccountLockState::Unlocked);
}

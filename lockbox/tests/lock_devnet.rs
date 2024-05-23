use std::str::FromStr;

use conjunto_core::{CommitFrequency, DelegationRecord};
use conjunto_lockbox::{AccountLockState, AccountLockStateProvider};
use conjunto_providers::{
    rpc_account_provider::RpcAccountProvider,
    rpc_provider_config::RpcProviderConfig,
};
use conjunto_test_tools::delegation_record_parser_stub::DelegationRecordParserStub;
use solana_sdk::{pubkey::Pubkey, system_program};

fn default_delegation_record() -> DelegationRecord {
    DelegationRecord {
        commit_frequency: CommitFrequency::Millis(1_000),
    }
}

#[tokio::test]
async fn test_known_delegation() {
    // NOTE: this test depends on these accounts being present on devnet
    // and properly locked

    let delegated_addr = "8k2V7EzQtNg38Gi9HK5ZtQYp1YpGKNGrMcuGa737gZX4";
    let delegated_id = Pubkey::from_str(delegated_addr).unwrap();

    let delegation_addr = "CkieZJmrj6dLhwteG69LSutpWwRHiDJY9S8ua7xJ7CRW";
    let delegation_id = Pubkey::from_str(delegation_addr).unwrap();

    let delegation_record = default_delegation_record();
    let mut delegation_record_parser = DelegationRecordParserStub::default();
    delegation_record_parser.set_next_record(delegation_record.clone());

    let lockstate_provider = AccountLockStateProvider::<
        RpcAccountProvider,
        DelegationRecordParserStub,
    >::new_with_parser(
        RpcProviderConfig::default(),
        delegation_record_parser,
    );

    let state = lockstate_provider
        .try_lockstate_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert_eq!(
        state,
        AccountLockState::Locked {
            delegated_id,
            delegation_pda: delegation_id,
            config: delegation_record.into(),
        }
    );
}

#[tokio::test]
async fn test_system_account_not_delegated() {
    let delegated_id = system_program::id();

    let lockstate_provider = AccountLockStateProvider::<
        RpcAccountProvider,
        DelegationRecordParserStub,
    >::new(RpcProviderConfig::default());

    let state = lockstate_provider
        .try_lockstate_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert!(matches!(state, AccountLockState::Unlocked { .. }));
}

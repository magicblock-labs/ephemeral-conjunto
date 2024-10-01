use conjunto_core::{
    delegation_inconsistency::DelegationInconsistency,
    delegation_record::{CommitFrequency, DelegationRecord},
    AccountProvider,
};
use conjunto_lockbox::{
    account_chain_snapshot_provider::AccountChainSnapshotProvider,
    account_chain_state::AccountChainState,
};
use conjunto_providers::{
    rpc_account_provider::RpcAccountProvider,
    rpc_provider_config::RpcProviderConfig,
};
use conjunto_test_tools::delegation_record_parser_stub::DelegationRecordParserStub;
use dlp::consts::DELEGATION_PROGRAM_ID;
use solana_sdk::{
    bpf_loader_upgradeable::get_program_data_address,
    {pubkey, pubkey::Pubkey},
};

fn dummy_delegation_record() -> DelegationRecord {
    DelegationRecord {
        authority: Pubkey::new_unique(),
        owner: Pubkey::new_unique(),
        delegation_slot: 0,
        commit_frequency: CommitFrequency::Millis(1_000),
    }
}

#[tokio::test]
async fn test_known_delegation() {
    // NOTE: this test depends on these accounts being present on devnet and properly locked
    let rpc_account_provider =
        RpcAccountProvider::new(RpcProviderConfig::devnet());

    let pubkey = pubkey!("8k2V7EzQtNg38Gi9HK5ZtQYp1YpGKNGrMcuGa737gZX4");

    let (at_slot, account) =
        rpc_account_provider.get_account(&pubkey).await.unwrap();

    let delegation_record = dummy_delegation_record();

    let mut delegation_record_parser = DelegationRecordParserStub::default();
    delegation_record_parser.set_next_record(delegation_record.clone());

    let account_chain_snapshot_provider = AccountChainSnapshotProvider::new(
        rpc_account_provider,
        delegation_record_parser,
    );

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(&pubkey)
        .await
        .unwrap();

    assert_eq!(chain_snapshot.pubkey, pubkey);
    assert!(chain_snapshot.at_slot >= at_slot);
    assert_eq!(
        chain_snapshot.chain_state,
        AccountChainState::Delegated {
            account: account.unwrap(),
            delegation_record,
        }
    );
}

#[tokio::test]
async fn test_delegation_program_as_data() {
    // NOTE: this test depends on devnet being up
    let rpc_account_provider =
        RpcAccountProvider::new(RpcProviderConfig::devnet());

    let pubkey = get_program_data_address(&DELEGATION_PROGRAM_ID);

    let (at_slot, account) =
        rpc_account_provider.get_account(&pubkey).await.unwrap();

    let delegation_record_parser = DelegationRecordParserStub::default();

    let account_chain_snapshot_provider = AccountChainSnapshotProvider::new(
        rpc_account_provider,
        delegation_record_parser,
    );

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(&pubkey)
        .await
        .unwrap();

    assert_eq!(chain_snapshot.pubkey, pubkey);
    assert!(chain_snapshot.at_slot >= at_slot);
    assert_eq!(
        chain_snapshot.chain_state,
        AccountChainState::Undelegated {
            account: account.unwrap(),
            delegation_inconsistency:
                DelegationInconsistency::AccountInvalidOwner,
        }
    );
}

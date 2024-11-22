use conjunto_core::{
    delegation_inconsistency::DelegationInconsistency,
    delegation_record::{CommitFrequency, DelegationRecord},
};
use conjunto_lockbox::{
    account_chain_snapshot::AccountChainSnapshot,
    account_chain_snapshot_provider::AccountChainSnapshotProvider,
    account_chain_state::AccountChainState,
};
use conjunto_test_tools::{
    account_provider_stub::AccountProviderStub,
    accounts::{
        account_owned_by_delegation_program, account_owned_by_system_program,
        account_with_data, delegated_account_ids,
    },
    delegation_record_parser_stub::DelegationRecordParserStub,
};
use solana_sdk::{
    account::Account, clock::Slot, pubkey::Pubkey, signature::Keypair,
    signer::Signer, system_program,
};

const EXPECTED_SLOT: Slot = 42;

fn dummy_delegation_record() -> DelegationRecord {
    DelegationRecord {
        authority: Pubkey::new_unique(),
        owner: Pubkey::new_unique(),
        delegation_slot: 0,
        commit_frequency: CommitFrequency::Millis(1_000),
    }
}

fn setup(
    accounts: Vec<(Pubkey, Account)>,
    delegation_record: Option<DelegationRecord>,
) -> AccountChainSnapshotProvider<AccountProviderStub, DelegationRecordParserStub>
{
    let mut account_provider = AccountProviderStub::default();
    account_provider.at_slot = EXPECTED_SLOT;
    for (pubkey, account) in accounts {
        account_provider.add(pubkey, account);
    }
    let mut delegation_record_parser = DelegationRecordParserStub::default();
    if let Some(record) = delegation_record {
        delegation_record_parser.set_next_record(record);
    }
    AccountChainSnapshotProvider::new(
        account_provider,
        delegation_record_parser,
    )
}

#[tokio::test]
async fn test_snapshot_account_does_not_exist() {
    let pubkey = Pubkey::new_unique();

    let account_chain_snapshot_provider = setup(vec![], None);

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(&pubkey, None)
        .await
        .unwrap();

    assert_eq!(
        chain_snapshot,
        AccountChainSnapshot {
            pubkey,
            at_slot: EXPECTED_SLOT,
            chain_state: AccountChainState::FeePayer {
                lamports: 0,
                owner: system_program::ID
            }
        }
    );
}

#[tokio::test]
async fn test_snapshot_account_with_lamports() {
    let pubkey = Keypair::new().pubkey();

    let mut account = account_owned_by_system_program();
    account.lamports = 42;

    let account_chain_snapshot_provider = setup(vec![(pubkey, account)], None);

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(&pubkey, None)
        .await
        .unwrap();

    assert_eq!(
        chain_snapshot,
        AccountChainSnapshot {
            pubkey,
            at_slot: EXPECTED_SLOT,
            chain_state: AccountChainState::FeePayer {
                lamports: 42,
                owner: system_program::ID
            }
        }
    );
}

#[tokio::test]
async fn test_snapshot_account_with_data() {
    let pubkey = Keypair::new().pubkey();

    let account = account_with_data();

    let account_chain_snapshot_provider =
        setup(vec![(pubkey, account.clone())], None);

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(&pubkey, None)
        .await
        .unwrap();

    assert_eq!(
        chain_snapshot,
        AccountChainSnapshot {
            pubkey,
            at_slot: EXPECTED_SLOT,
            chain_state: AccountChainState::Undelegated {
                account,
                delegation_inconsistency:
                    DelegationInconsistency::AccountInvalidOwner
            }
        }
    );
}

#[tokio::test]
async fn test_snapshot_delegated() {
    let (pubkey, delegation_record_pubkey) = delegated_account_ids();

    let account = account_owned_by_delegation_program();
    let delegation_record = dummy_delegation_record();

    let account_chain_snapshot_provider = setup(
        vec![
            (pubkey, account.clone()),
            (
                delegation_record_pubkey,
                account_owned_by_delegation_program(),
            ),
        ],
        Some(delegation_record.clone()),
    );

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(&pubkey, None)
        .await
        .unwrap();

    assert_eq!(
        chain_snapshot,
        AccountChainSnapshot {
            pubkey,
            at_slot: EXPECTED_SLOT,
            chain_state: AccountChainState::Delegated {
                account,
                delegation_record,
            }
        }
    );
}

#[tokio::test]
async fn test_snapshot_account_invalid_owner() {
    let (pubkey, delegation_record_pubkey) = delegated_account_ids();

    let account = account_with_data();
    let delegation_record = dummy_delegation_record();

    let account_chain_snapshot_provider = setup(
        vec![
            (pubkey, account.clone()),
            (
                delegation_record_pubkey,
                account_owned_by_delegation_program(),
            ),
        ],
        Some(delegation_record.clone()),
    );

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(&pubkey, None)
        .await
        .unwrap();

    assert_eq!(
        chain_snapshot,
        AccountChainSnapshot {
            pubkey,
            at_slot: EXPECTED_SLOT,
            chain_state: AccountChainState::Undelegated {
                account,
                delegation_inconsistency:
                    DelegationInconsistency::AccountInvalidOwner,
            }
        }
    );
}

#[tokio::test]
async fn test_snapshot_delegation_record_not_found() {
    let (pubkey, _delegation_record_pda) = delegated_account_ids();

    let account = account_owned_by_delegation_program();

    let account_chain_snapshot_provider =
        setup(vec![(pubkey, account.clone())], None);

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(&pubkey, None)
        .await
        .unwrap();

    assert_eq!(
        chain_snapshot,
        AccountChainSnapshot {
            pubkey,
            at_slot: EXPECTED_SLOT,
            chain_state: AccountChainState::Undelegated {
                account,
                delegation_inconsistency:
                    DelegationInconsistency::DelegationRecordNotFound,
            }
        }
    );
}

#[tokio::test]
async fn test_snapshot_delegation_record_invalid_owner() {
    let (pubkey, delegation_record_pubkey) = delegated_account_ids();

    let account = account_owned_by_delegation_program();
    let delegation_record = dummy_delegation_record();

    let account_chain_snapshot_provider = setup(
        vec![
            (pubkey, account_owned_by_delegation_program()),
            (delegation_record_pubkey, account_with_data()),
        ],
        Some(delegation_record.clone()),
    );

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(&pubkey, None)
        .await
        .unwrap();

    assert_eq!(
        chain_snapshot,
        AccountChainSnapshot {
            pubkey,
            at_slot: EXPECTED_SLOT,
            chain_state: AccountChainState::Undelegated {
                account,
                delegation_inconsistency:
                    DelegationInconsistency::DelegationRecordInvalidOwner,
            }
        }
    );
}

#[tokio::test]
async fn test_snapshot_delegation_record_data_invalid() {
    let (pubkey, delegation_record_pubkey) = delegated_account_ids();

    let account = account_owned_by_delegation_program();

    let account_chain_snapshot_provider = setup(
        vec![
            (pubkey, account_owned_by_delegation_program()),
            (
                delegation_record_pubkey,
                account_owned_by_delegation_program(),
            ),
        ],
        None,
    );

    let chain_snapshot = account_chain_snapshot_provider
        .try_fetch_chain_snapshot_of_pubkey(&pubkey, None)
        .await
        .unwrap();

    assert_eq!(
        chain_snapshot,
        AccountChainSnapshot {
            pubkey,
            at_slot: EXPECTED_SLOT,
            chain_state: AccountChainState::Undelegated {
                account,
                delegation_inconsistency:
                    DelegationInconsistency::DelegationRecordDataInvalid(
                        "Failed to parse account data".to_string()
                    ),
            }
        }
    );
}

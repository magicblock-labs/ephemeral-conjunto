use conjunto_core::delegation_inconsistency::DelegationInconsistency;
use conjunto_lockbox::{
    account_chain_snapshot::AccountChainSnapshot,
    account_chain_state::AccountChainState,
};
use conjunto_test_tools::accounts::{
    account_owned_by_delegation_program, account_with_data,
};
use conjunto_transwise::{
    transaction_accounts_snapshot::TransactionAccountsSnapshot,
    transaction_accounts_validator::{
        TransactionAccountsValidator, TransactionAccountsValidatorImpl,
    },
    AccountChainSnapshotShared, CommitFrequency, DelegationRecord,
};
use solana_sdk::{pubkey::Pubkey, system_program};

fn transaction_accounts_validator() -> TransactionAccountsValidatorImpl {
    TransactionAccountsValidatorImpl {}
}

fn chain_snapshot_feepayer() -> AccountChainSnapshotShared {
    AccountChainSnapshot {
        pubkey: Pubkey::new_unique(),
        at_slot: 42,
        chain_state: AccountChainState::FeePayer {
            lamports: 42,
            owner: system_program::ID,
        },
    }
    .into()
}
fn chain_snapshot_undelegated() -> AccountChainSnapshotShared {
    AccountChainSnapshot {
        pubkey: Pubkey::new_unique(),
        at_slot: 42,
        chain_state: AccountChainState::Undelegated {
            account: account_with_data(),
            delegation_inconsistency:
                DelegationInconsistency::AccountInvalidOwner,
        },
    }
    .into()
}
fn chain_snapshot_delegated() -> AccountChainSnapshotShared {
    AccountChainSnapshot {
        pubkey: Pubkey::new_unique(),
        at_slot: 42,
        chain_state: AccountChainState::Delegated {
            account: account_owned_by_delegation_program(),
            delegation_record: DelegationRecord {
                authority: Pubkey::new_unique(),
                owner: Pubkey::new_unique(),
                delegation_slot: 0,
                commit_frequency: CommitFrequency::Millis(1_000),
            },
        },
    }
    .into()
}

#[test]
fn test_two_readonly_undelegated_and_two_writable_delegated_and_feepayers() {
    let readonly_undelegated1 = chain_snapshot_undelegated();
    let readonly_undelegated2 = chain_snapshot_undelegated();
    let readonly_feepayer = chain_snapshot_feepayer();
    let writable_delegated1 = chain_snapshot_delegated();
    let writable_delegated2 = chain_snapshot_delegated();
    let writable_feepayer = chain_snapshot_feepayer();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_feepayer.pubkey,
                readonly: vec![
                    readonly_undelegated1,
                    readonly_undelegated2,
                    readonly_feepayer,
                ],
                writable: vec![
                    writable_delegated1,
                    writable_delegated2,
                    writable_feepayer,
                ],
            },
        );

    // This is a fairly typical case that should work (feepayer and writables are in good condition)
    assert!(result.is_ok());
}

#[test]
fn test_empty_transaction_accounts() {
    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: Pubkey::new_unique(),
                readonly: vec![],
                writable: vec![],
            },
        );

    // No data writables, so it's fine (solana will deny the transaction tho, because no payer)
    assert!(result.is_ok());
}

#[test]
fn test_only_one_readonly_undelegated() {
    let readonly_undelegated = chain_snapshot_undelegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: Pubkey::new_unique(),
                readonly: vec![readonly_undelegated],
                writable: vec![],
            },
        );

    // No data writables, so it's fine (solana will deny the transaction tho, because no payer)
    assert!(result.is_ok());
}

#[test]
fn test_only_one_writable_delegated() {
    let writable_delegated = chain_snapshot_delegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: Pubkey::new_unique(),
                readonly: vec![],
                writable: vec![writable_delegated],
            },
        );

    // No data writables, so it's fine (solana will deny the transaction tho, because no payer)
    assert!(result.is_ok());
}

#[test]
fn test_only_one_writable_feepayer() {
    let writable_feepayer = chain_snapshot_feepayer();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: Pubkey::new_unique(),
                readonly: vec![],
                writable: vec![writable_feepayer],
            },
        );

    // No data writables, so it's fine (solana will deny the transaction tho, because no payer)
    assert!(result.is_ok());
}

#[test]
fn test_only_one_readable_undelegated_as_payer() {
    let readable_undelegated = chain_snapshot_undelegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: readable_undelegated.pubkey,
                readonly: vec![readable_undelegated],
                writable: vec![],
            },
        );

    // No data writables, so it's fine (solana will deny the transaction tho, because invalid payer)
    assert!(result.is_ok());
}

#[test]
fn test_only_one_writable_undelegated_as_payer_fail() {
    let writable_undelegated = chain_snapshot_undelegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_undelegated.pubkey,
                readonly: vec![],
                writable: vec![writable_undelegated],
            },
        );

    // This transaction's payer is data, that's not good, we should NOT allow this
    assert!(result.is_err());
}

#[test]
fn test_only_one_writable_delegated_as_payer() {
    let writable_delegated = chain_snapshot_delegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_delegated.pubkey,
                readonly: vec![],
                writable: vec![writable_delegated],
            },
        );

    // No data writables, so it's fine (solana will deny the transaction tho, because invalid payer)
    assert!(result.is_ok());
}

#[test]
fn test_only_one_writable_feepayer_as_payer() {
    let writable_feepayer = chain_snapshot_feepayer();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_feepayer.pubkey,
                readonly: vec![],
                writable: vec![writable_feepayer],
            },
        );

    // Because there is a payer a wallet, this should work fine
    assert!(result.is_ok());
}

#[test]
fn test_one_readonly_undelegated_and_writable_feepayer_as_payer() {
    let readonly_undelegated = chain_snapshot_undelegated();
    let writable_feepayer = chain_snapshot_feepayer();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_feepayer.pubkey,
                readonly: vec![readonly_undelegated],
                writable: vec![writable_feepayer],
            },
        );

    // This should work, this is a fairly common case
    assert!(result.is_ok());
}

#[test]
fn test_one_readonly_undelegated_and_one_writable_delegated_and_feepayer() {
    let readonly_undelegated = chain_snapshot_undelegated();
    let writable_delegated = chain_snapshot_delegated();
    let writable_feepayer = chain_snapshot_feepayer();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: Pubkey::new_unique(),
                readonly: vec![readonly_undelegated],
                writable: vec![writable_delegated, writable_feepayer],
            },
        );

    // This should work, this is a fairly common case
    assert!(result.is_ok());
}

#[test]
fn test_one_readonly_undelegated_and_one_writable_undelegated_and_payer_fail() {
    let readonly_undelegated = chain_snapshot_undelegated();
    let writable_undelegated = chain_snapshot_undelegated();
    let writable_feepayer = chain_snapshot_feepayer();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: Pubkey::new_unique(),
                readonly: vec![readonly_undelegated],
                writable: vec![writable_undelegated, writable_feepayer],
            },
        );

    // Any writable data should fail
    assert!(result.is_err());
}

#[test]
fn test_one_readonly_undelegated_and_one_writable_undelegated_as_payer_fail() {
    let readonly_undelegated = chain_snapshot_undelegated();
    let writable_undelegated = chain_snapshot_undelegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_undelegated.pubkey,
                readonly: vec![readonly_undelegated],
                writable: vec![writable_undelegated],
            },
        );

    // Payer is data and writable, which is wrong
    assert!(result.is_err());
}

#[test]
fn test_one_writable_undelegated_and_writable_feepayer_as_payer_fail() {
    let writable_undelegated = chain_snapshot_undelegated();
    let writable_feepayer = chain_snapshot_feepayer();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_feepayer.pubkey,
                readonly: vec![],
                writable: vec![writable_undelegated, writable_feepayer],
            },
        );

    // Even if the payer is correct, we have a data account as writable so this should not work
    assert!(result.is_err());
}

#[test]
fn test_one_of_each_valid_type() {
    let readonly_undelegated = chain_snapshot_undelegated();
    let readonly_delegated = chain_snapshot_delegated();
    let readonly_feepayer = chain_snapshot_feepayer();

    let writable_delegated = chain_snapshot_delegated();
    let writable_feepayer = chain_snapshot_feepayer();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_feepayer.pubkey,
                readonly: vec![
                    readonly_undelegated,
                    readonly_delegated,
                    readonly_feepayer,
                ],
                writable: vec![writable_delegated, writable_feepayer],
            },
        );

    // This should work just right in strict mode
    assert!(result.is_ok());
}

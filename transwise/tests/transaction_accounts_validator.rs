use conjunto_lockbox::{
    account_chain_snapshot::AccountChainSnapshot,
    account_chain_state::AccountChainState,
};
use conjunto_test_tools::accounts::{
    account_owned_by_delegation_program, account_owned_by_system_program,
};
use conjunto_transwise::{
    transaction_accounts_snapshot::TransactionAccountsSnapshot,
    transaction_accounts_validator::{
        TransactionAccountsValidator, TransactionAccountsValidatorImpl,
    },
    AccountChainSnapshotShared, CommitFrequency, DelegationRecord,
};
use solana_sdk::pubkey::Pubkey;

fn transaction_accounts_validator() -> TransactionAccountsValidatorImpl {
    TransactionAccountsValidatorImpl {}
}

fn chain_snapshot_delegated() -> AccountChainSnapshotShared {
    AccountChainSnapshot {
        pubkey: Pubkey::new_unique(),
        at_slot: 42,
        chain_state: AccountChainState::Delegated {
            account: account_owned_by_delegation_program(),
            delegation_pda: Pubkey::new_unique(),
            delegation_record: DelegationRecord {
                commit_frequency: CommitFrequency::Millis(1_000),
                owner: Pubkey::new_unique(),
                delegation_slot: 0,
            },
        },
    }
    .into()
}
fn chain_snapshot_undelegated() -> AccountChainSnapshotShared {
    AccountChainSnapshot {
        pubkey: Pubkey::new_unique(),
        at_slot: 42,
        chain_state: AccountChainState::Undelegated {
            account: account_owned_by_system_program(),
        },
    }
    .into()
}
fn chain_snapshot_new_account() -> AccountChainSnapshotShared {
    AccountChainSnapshot {
        pubkey: Pubkey::new_unique(),
        at_slot: 42,
        chain_state: AccountChainState::NewAccount,
    }
    .into()
}
fn chain_snapshot_inconsistent() -> AccountChainSnapshotShared {
    AccountChainSnapshot {
        pubkey: Pubkey::new_unique(),
        at_slot: 42,
        chain_state: AccountChainState::Inconsistent {
            account: account_owned_by_system_program(),
            delegation_pda: Pubkey::new_unique(),
            delegation_inconsistencies: vec![],
        },
    }
    .into()
}

#[test]
fn test_two_readonly_undelegated_and_two_writable_delegated_and_payer() {
    let readonly_undelegated1 = chain_snapshot_undelegated();
    let readonly_undelegated2 = chain_snapshot_undelegated();
    let writable_delegated1 = chain_snapshot_delegated();
    let writable_delegated2 = chain_snapshot_delegated();
    let writable_undelegated_payer = chain_snapshot_undelegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_undelegated_payer.pubkey.clone(),
                readonly: vec![readonly_undelegated1, readonly_undelegated2],
                writable: vec![
                    writable_delegated1,
                    writable_delegated2,
                    writable_undelegated_payer,
                ],
            },
        );

    // This is a fairly typical case that should work (payer and writables are in good condition)
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

    // Empty transactions are missing a payer, we allow that for now
    assert!(result.is_ok());
}

#[test]
fn test_only_one_readonly_undelegated_non_payer() {
    let readonly_undelegated = chain_snapshot_undelegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: Pubkey::new_unique(),
                readonly: vec![readonly_undelegated],
                writable: vec![],
            },
        );

    // This transaction is missing a payer, we allow that for now
    assert!(result.is_ok());
}

#[test]
fn test_only_one_writable_delegated_non_payer() {
    let writable_delegated = chain_snapshot_delegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: Pubkey::new_unique(),
                readonly: vec![],
                writable: vec![writable_delegated],
            },
        );

    // This transaction is missing a payer, we allow that for now
    assert!(result.is_ok());
}

#[test]
fn test_only_one_readable_undelegated_payer() {
    let readable_undelegated_payer = chain_snapshot_undelegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: readable_undelegated_payer.pubkey.clone(),
                readonly: vec![readable_undelegated_payer],
                writable: vec![],
            },
        );

    // This transaction's payer is readonly, we allow that for now
    assert!(result.is_ok());
}

#[test]
fn test_only_one_writable_undelegated_payer() {
    let writable_undelegated_payer = chain_snapshot_undelegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_undelegated_payer.pubkey.clone(),
                readonly: vec![],
                writable: vec![writable_undelegated_payer],
            },
        );

    // Because there is one writable and it's a payer, this should work even when payer is not delegated
    assert!(result.is_ok());
}

#[test]
fn test_only_one_writable_new_account_payer_fail() {
    let writable_new_account_payer = chain_snapshot_new_account();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_new_account_payer.pubkey.clone(),
                readonly: vec![],
                writable: vec![writable_new_account_payer],
            },
        );

    // Because there is a payer new account, this should not work
    assert!(result.is_err());
}

#[test]
fn test_only_one_writable_inconsistent_payer_fail() {
    let writable_inconsistent_payer = chain_snapshot_inconsistent();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_inconsistent_payer.pubkey.clone(),
                readonly: vec![],
                writable: vec![writable_inconsistent_payer],
            },
        );

    // Because there is an inconsistent delegation record, this should fail, even if its the payer
    assert!(result.is_err());
}

#[test]
fn test_one_readonly_undelegated_and_payer() {
    let readonly_undelegated = chain_snapshot_undelegated();
    let writable_undelegated_payer = chain_snapshot_undelegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_undelegated_payer.pubkey.clone(),
                readonly: vec![readonly_undelegated],
                writable: vec![writable_undelegated_payer],
            },
        );

    // Even if it's a writable undelegated, it should work because that's our payer
    assert!(result.is_ok());
}

#[test]
fn test_one_readonly_undelegated_and_one_writable_undelegated_and_payer_fail() {
    let readonly_undelegated = chain_snapshot_undelegated();
    let writable_undelegated = chain_snapshot_undelegated();
    let writable_undelegated_payer = chain_snapshot_undelegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: Pubkey::new_unique(),
                readonly: vec![readonly_undelegated],
                writable: vec![
                    writable_undelegated,
                    writable_undelegated_payer,
                ],
            },
        );

    // Because there is a non-payer writable undelegated, this should not work
    assert!(result.is_err());
}

#[test]
fn test_one_readonly_undelegated_and_one_writable_inconsistent_and_payer_fail()
{
    let readonly_undelegated = chain_snapshot_undelegated();
    let writable_inconsistent = chain_snapshot_inconsistent();
    let writable_undelegated_payer = chain_snapshot_undelegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: Pubkey::new_unique(),
                readonly: vec![readonly_undelegated],
                writable: vec![
                    writable_inconsistent,
                    writable_undelegated_payer,
                ],
            },
        );

    // Any writable inconsistent should fail
    assert!(result.is_err());
}

#[test]
fn test_one_readonly_new_account_and_payer() {
    let readonly_new_account = chain_snapshot_new_account();
    let writable_undelegated_payer = chain_snapshot_undelegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_undelegated_payer.pubkey.clone(),
                readonly: vec![readonly_new_account],
                writable: vec![writable_undelegated_payer],
            },
        );

    // While this is a new account, it's a readonly so we don't need to write to it, so it's valid
    assert!(result.is_ok());
}

#[test]
fn test_one_writable_new_account_and_payer_fail() {
    let writable_new_account = chain_snapshot_new_account();
    let writable_undelegated_payer = chain_snapshot_undelegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_undelegated_payer.pubkey.clone(),
                readonly: vec![],
                writable: vec![
                    writable_new_account,
                    writable_undelegated_payer,
                ],
            },
        );

    // while the rest of the transaction is valid, because we have a writable new account, it should fail
    assert!(result.is_err());
}

#[test]
fn test_one_of_each_valid_type() {
    let readonly_new_account = chain_snapshot_new_account();
    let readonly_undelegated = chain_snapshot_undelegated();
    let readonly_delegated = chain_snapshot_delegated();
    let readonly_inconsistent = chain_snapshot_inconsistent();

    let writable_delegated = chain_snapshot_delegated();
    let writable_undelegated_payer = chain_snapshot_undelegated();

    let result = transaction_accounts_validator()
        .validate_ephemeral_transaction_accounts(
            &TransactionAccountsSnapshot {
                payer: writable_undelegated_payer.pubkey.clone(),
                readonly: vec![
                    readonly_new_account,
                    readonly_undelegated,
                    readonly_delegated,
                    readonly_inconsistent,
                ],
                writable: vec![writable_delegated, writable_undelegated_payer],
            },
        );

    // This should work just right in strict mode
    assert!(result.is_ok());
}

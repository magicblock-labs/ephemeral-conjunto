use std::str::FromStr;

use conjunto_addresses::consts::DELEGATION_PROGRAM_ID;
use conjunto_lockbox::{
    AccountLockState, AccountLockStateProvider, LockInconsistency,
};
use solana_sdk::{account::Account, pubkey::Pubkey, system_program};

use crate::utils::AccountProviderStub;

mod utils;

fn account_owned_by_delegation_program() -> Account {
    Account {
        owner: DELEGATION_PROGRAM_ID,
        ..Account::default()
    }
}

fn account_owned_by_system_program() -> Account {
    Account {
        owner: system_program::id(),
        ..Account::default()
    }
}

fn account_ids() -> (Pubkey, Pubkey, Pubkey) {
    let delegated_addr = "8k2V7EzQtNg38Gi9HK5ZtQYp1YpGKNGrMcuGa737gZX4";
    let delegated_id = Pubkey::from_str(delegated_addr).unwrap();

    let buffer_addr = "E8NdkAGLLC3qnvphsXhqkjkXpRkdoiDpicSTTQJySVtG";
    let buffer_pda = Pubkey::from_str(buffer_addr).unwrap();

    let delegation_addr = "CkieZJmrj6dLhwteG69LSutpWwRHiDJY9S8ua7xJ7CRW";
    let delegation_pda = Pubkey::from_str(delegation_addr).unwrap();

    (delegated_id, buffer_pda, delegation_pda)
}

fn setup(
    accounts: Vec<(Pubkey, Account)>,
) -> AccountLockStateProvider<AccountProviderStub> {
    let mut account_provider = AccountProviderStub::default();
    for (pubkey, account) in accounts {
        account_provider.add(pubkey, account);
    }
    AccountLockStateProvider::with_provider(account_provider)
}

#[tokio::test]
async fn test_delegate_properly_locked() {
    let (delegated_id, buffer_pda, delegation_pda) = account_ids();
    let lockstate_provider = setup(vec![
        (delegated_id, account_owned_by_delegation_program()),
        (buffer_pda, account_owned_by_delegation_program()),
        (delegation_pda, account_owned_by_delegation_program()),
    ]);

    let state = lockstate_provider
        .try_lockstate_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert_eq!(
        state,
        AccountLockState::Locked {
            delegated_id,
            buffer_pda,
            delegation_pda
        }
    );
}

#[tokio::test]
async fn test_delegate_unlocked() {
    let (delegated_id, buffer_pda, delegation_pda) = account_ids();
    let lockstate_provider = setup(vec![
        (delegated_id, account_owned_by_system_program()),
        // The other accounts don't matter since we don't check them if no lock is present
        (buffer_pda, account_owned_by_delegation_program()),
        (delegation_pda, account_owned_by_delegation_program()),
    ]);

    let state = lockstate_provider
        .try_lockstate_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert_eq!(state, AccountLockState::Unlocked);
}

#[tokio::test]
async fn test_delegate_not_found() {
    let (delegated_id, buffer_pda, delegation_pda) = account_ids();
    let lockstate_provider = setup(vec![
        // The other accounts don't matter since we don't check them if delegated
        // account is missing
        (buffer_pda, account_owned_by_delegation_program()),
        (delegation_pda, account_owned_by_delegation_program()),
    ]);

    let state = lockstate_provider
        .try_lockstate_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert_eq!(
        state,
        AccountLockState::Inconsistent {
            delegated_id,
            buffer_pda,
            delegation_pda,
            inconsistencies: vec![LockInconsistency::DelegateAccountNotFound]
        }
    );
}

#[tokio::test]
async fn test_delegate_missing_buffer_account() {
    let (delegated_id, buffer_pda, delegation_pda) = account_ids();

    let lockstate_provider = setup(vec![
        (delegated_id, account_owned_by_delegation_program()),
        (delegation_pda, account_owned_by_delegation_program()),
    ]);

    let state = lockstate_provider
        .try_lockstate_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert_eq!(
        state,
        AccountLockState::Inconsistent {
            delegated_id,
            buffer_pda,
            delegation_pda,
            inconsistencies: vec![LockInconsistency::BufferAccountNotFound]
        }
    );
}

#[tokio::test]
async fn test_delegate_missing_delegate_account() {
    let (delegated_id, buffer_pda, delegation_pda) = account_ids();

    let lockstate_provider = setup(vec![
        (delegated_id, account_owned_by_delegation_program()),
        (buffer_pda, account_owned_by_delegation_program()),
    ]);

    let state = lockstate_provider
        .try_lockstate_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert_eq!(
        state,
        AccountLockState::Inconsistent {
            delegated_id,
            buffer_pda,
            delegation_pda,
            inconsistencies: vec![LockInconsistency::DelegationAccountNotFound]
        }
    );
}

#[tokio::test]
async fn test_delegate_missing_buffer_and_delegation_accounts() {
    let (delegated_id, buffer_pda, delegation_pda) = account_ids();

    let lockstate_provider =
        setup(vec![(delegated_id, account_owned_by_delegation_program())]);

    let state = lockstate_provider
        .try_lockstate_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert_eq!(
        state,
        AccountLockState::Inconsistent {
            delegated_id,
            buffer_pda,
            delegation_pda,
            inconsistencies: vec![
                LockInconsistency::BufferAccountNotFound,
                LockInconsistency::DelegationAccountNotFound
            ]
        }
    );
}

#[tokio::test]
async fn test_delegate_buffer_not_owned_by_delegate_program() {
    let (delegated_id, buffer_pda, delegation_pda) = account_ids();
    let lockstate_provider = setup(vec![
        (delegated_id, account_owned_by_delegation_program()),
        (buffer_pda, account_owned_by_system_program()),
        (delegation_pda, account_owned_by_delegation_program()),
    ]);

    let state = lockstate_provider
        .try_lockstate_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert_eq!(
        state,
        AccountLockState::Inconsistent {
            delegated_id,
            buffer_pda,
            delegation_pda,
            inconsistencies: vec![LockInconsistency::BufferAccountInvalidOwner]
        }
    );
}

#[tokio::test]
async fn test_delegate_delegation_not_owned_by_delegate_program() {
    let (delegated_id, buffer_pda, delegation_pda) = account_ids();
    let lockstate_provider = setup(vec![
        (delegated_id, account_owned_by_delegation_program()),
        (buffer_pda, account_owned_by_delegation_program()),
        (delegation_pda, account_owned_by_system_program()),
    ]);

    let state = lockstate_provider
        .try_lockstate_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert_eq!(
        state,
        AccountLockState::Inconsistent {
            delegated_id,
            buffer_pda,
            delegation_pda,
            inconsistencies: vec![
                LockInconsistency::DelegationAccountInvalidOwner
            ]
        }
    );
}

#[tokio::test]
async fn test_delegate_buffer_missing_and_delegation_not_owned_by_delegate_program(
) {
    let (delegated_id, buffer_pda, delegation_pda) = account_ids();
    let lockstate_provider = setup(vec![
        (delegated_id, account_owned_by_delegation_program()),
        (delegation_pda, account_owned_by_system_program()),
    ]);

    let state = lockstate_provider
        .try_lockstate_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert_eq!(
        state,
        AccountLockState::Inconsistent {
            delegated_id,
            buffer_pda,
            delegation_pda,
            inconsistencies: vec![
                LockInconsistency::BufferAccountNotFound,
                LockInconsistency::DelegationAccountInvalidOwner
            ]
        }
    );
}

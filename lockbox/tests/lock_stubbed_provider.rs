use conjunto_lockbox::{
    AccountLockState, AccountLockStateProvider, LockInconsistency,
};
use conjunto_test_tools::{
    account_provider_stub::AccountProviderStub,
    accounts::{
        account_owned_by_delegation_program, account_owned_by_system_program,
        delegated_account_ids,
    },
};
use solana_sdk::{account::Account, pubkey::Pubkey};

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
    let (delegated_id, delegation_pda) = delegated_account_ids();
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
        AccountLockState::Locked {
            delegated_id,
            delegation_pda
        }
    );
}

#[tokio::test]
async fn test_delegate_unlocked() {
    let (delegated_id, delegation_pda) = delegated_account_ids();
    let lockstate_provider = setup(vec![
        (delegated_id, account_owned_by_system_program()),
        // The other accounts don't matter since we don't check them if no lock is present
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
    let (delegated_id, delegation_pda) = delegated_account_ids();
    let lockstate_provider = setup(vec![
        // The other accounts don't matter since we don't check them if delegated
        // account is missing
        (delegation_pda, account_owned_by_delegation_program()),
    ]);

    let state = lockstate_provider
        .try_lockstate_of_pubkey(&delegated_id)
        .await
        .unwrap();

    assert_eq!(state, AccountLockState::NewAccount);
}

#[tokio::test]
async fn test_delegate_missing_delegate_account() {
    let (delegated_id, delegation_pda) = delegated_account_ids();

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
            delegation_pda,
            inconsistencies: vec![LockInconsistency::DelegationAccountNotFound]
        }
    );
}

#[tokio::test]
async fn test_delegate_delegation_not_owned_by_delegate_program() {
    let (delegated_id, delegation_pda) = delegated_account_ids();
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
            delegation_pda,
            inconsistencies: vec![
                LockInconsistency::DelegationAccountInvalidOwner
            ]
        }
    );
}

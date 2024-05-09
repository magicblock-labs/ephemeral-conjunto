use conjunto_lockbox::AccountLockStateProvider;
use conjunto_test_tools::{
    account_provider_stub::AccountProviderStub,
    accounts::{
        account_owned_by_delegation_program, account_owned_by_system_program,
        delegated_account_ids,
    },
    transaction_accounts_holder_stub::TransactionAccountsHolderStub,
};
use conjunto_transwise::trans_account_meta::TransAccountMetas;
use solana_sdk::{account::Account, pubkey::Pubkey};

fn setup_lockstate_provider(
    accounts: Vec<(Pubkey, Account)>,
) -> AccountLockStateProvider<AccountProviderStub> {
    let mut account_provider = AccountProviderStub::default();
    for (pubkey, account) in accounts {
        account_provider.add(pubkey, account);
    }
    AccountLockStateProvider::with_provider(account_provider)
}

#[tokio::test]
async fn test_account_meta_one_properly_locked_writable_and_one_readonly() {
    let (delegated_id, buffer_pda, delegation_pda) = delegated_account_ids();
    let lockstate_provider = setup_lockstate_provider(vec![
        (delegated_id, account_owned_by_delegation_program()),
        (buffer_pda, account_owned_by_delegation_program()),
        (delegation_pda, account_owned_by_delegation_program()),
    ]);
    let readonly_id = Pubkey::new_from_array([4u8; 32]);

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![readonly_id],
        writable: vec![delegated_id],
    };

    let account_metas = TransAccountMetas::from_accounts_holder(
        &acc_holder,
        &lockstate_provider,
    )
    .await
    .unwrap();
    let endpoint = account_metas.into_endpoint();

    eprintln!("{:#?}", endpoint);
    assert!(endpoint.is_ephemeral());
}

#[tokio::test]
async fn test_account_meta_one_properly_locked_writable_and_one_unlocked_writable(
) {
    let (delegated_id, buffer_pda, delegation_pda) = delegated_account_ids();
    let writable_id = Pubkey::new_from_array([4u8; 32]);
    let lockstate_provider = setup_lockstate_provider(vec![
        (delegated_id, account_owned_by_delegation_program()),
        (buffer_pda, account_owned_by_delegation_program()),
        (delegation_pda, account_owned_by_delegation_program()),
        (writable_id, account_owned_by_system_program()),
    ]);

    let acc_holder = TransactionAccountsHolderStub {
        writable: vec![delegated_id, writable_id],
        ..Default::default()
    };

    let account_metas = TransAccountMetas::from_accounts_holder(
        &acc_holder,
        &lockstate_provider,
    )
    .await
    .unwrap();
    let endpoint = account_metas.into_endpoint();

    eprintln!("{:#?}", endpoint);
    assert!(endpoint.is_unroutable());
}

#[tokio::test]
async fn test_account_meta_one_improperly_locked_writable_and_one_readonly() {
    let (delegated_id, buffer_pda, _) = delegated_account_ids();
    let lockstate_provider = setup_lockstate_provider(vec![
        (delegated_id, account_owned_by_delegation_program()),
        (buffer_pda, account_owned_by_delegation_program()),
        // Missing delegation account
    ]);
    let readonly_id = Pubkey::new_from_array([4u8; 32]);

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![readonly_id],
        writable: vec![delegated_id],
    };

    let endpoint = TransAccountMetas::from_accounts_holder(
        &acc_holder,
        &lockstate_provider,
    )
    .await
    .unwrap()
    .into_endpoint();

    eprintln!("{:#?}", endpoint);
    assert!(endpoint.is_unroutable());
}

#[tokio::test]
async fn test_account_meta_one_properly_locked_writable_and_one_new_writable() {
    let (delegated_id, buffer_pda, delegation_pda) = delegated_account_ids();
    let lockstate_provider = setup_lockstate_provider(vec![
        (delegated_id, account_owned_by_delegation_program()),
        (buffer_pda, account_owned_by_delegation_program()),
        (delegation_pda, account_owned_by_delegation_program()),
    ]);
    let new_writable_id = Pubkey::new_from_array([4u8; 32]);

    let acc_holder = TransactionAccountsHolderStub {
        writable: vec![delegated_id, new_writable_id],
        ..Default::default()
    };

    let endpoint = TransAccountMetas::from_accounts_holder(
        &acc_holder,
        &lockstate_provider,
    )
    .await
    .unwrap()
    .into_endpoint();

    eprintln!("{:#?}", endpoint);
    assert!(endpoint.is_ephemeral());
}

#[tokio::test]
async fn test_account_meta_one_new_writable() {
    let lockstate_provider = setup_lockstate_provider(vec![]);
    let new_writable_id = Pubkey::new_from_array([4u8; 32]);

    let acc_holder = TransactionAccountsHolderStub {
        writable: vec![new_writable_id],
        ..Default::default()
    };

    let endpoint = TransAccountMetas::from_accounts_holder(
        &acc_holder,
        &lockstate_provider,
    )
    .await
    .unwrap()
    .into_endpoint();

    eprintln!("{:#?}", endpoint);
    assert!(endpoint.is_ephemeral());
}

#[tokio::test]
async fn test_account_meta_one_unlocked_writable_two_readonlys() {
    let unlocked_writable_id = Pubkey::new_from_array([4u8; 32]);
    let lockstate_provider = setup_lockstate_provider(vec![(
        unlocked_writable_id,
        account_owned_by_system_program(),
    )]);
    let readonly1 = Pubkey::new_from_array([4u8; 32]);
    let readonly2 = Pubkey::new_from_array([5u8; 32]);

    let acc_holder = TransactionAccountsHolderStub {
        writable: vec![unlocked_writable_id],
        readonly: vec![readonly1, readonly2],
    };

    let endpoint = TransAccountMetas::from_accounts_holder(
        &acc_holder,
        &lockstate_provider,
    )
    .await
    .unwrap()
    .into_endpoint();

    eprintln!("{:#?}", endpoint);
    assert!(endpoint.is_chain());
}

#[tokio::test]
async fn test_account_meta_two_readonlys() {
    let lockstate_provider = setup_lockstate_provider(vec![]);
    let readonly1 = Pubkey::new_from_array([4u8; 32]);
    let readonly2 = Pubkey::new_from_array([5u8; 32]);

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![readonly1, readonly2],
        ..Default::default()
    };

    let endpoint = TransAccountMetas::from_accounts_holder(
        &acc_holder,
        &lockstate_provider,
    )
    .await
    .unwrap()
    .into_endpoint();

    eprintln!("{:#?}", endpoint);
    assert!(endpoint.is_unroutable());
}

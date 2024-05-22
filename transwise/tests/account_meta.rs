use conjunto_core::DelegationRecord;
use conjunto_lockbox::AccountLockStateProvider;
use conjunto_test_tools::{
    account_provider_stub::AccountProviderStub,
    accounts::{
        account_owned_by_delegation_program, account_owned_by_system_program,
        delegated_account_ids,
    },
    delegation_record_parser_stub::DelegationRecordParserStub,
    transaction_accounts_holder_stub::TransactionAccountsHolderStub,
};
use conjunto_transwise::trans_account_meta::TransAccountMetas;
use solana_sdk::{account::Account, pubkey::Pubkey};

fn setup_lockstate_provider(
    accounts: Vec<(Pubkey, Account)>,
    record: Option<DelegationRecord>,
) -> AccountLockStateProvider<AccountProviderStub, DelegationRecordParserStub> {
    let mut account_provider = AccountProviderStub::default();
    for (pubkey, account) in accounts {
        account_provider.add(pubkey, account);
    }
    let delegation_record_parser = DelegationRecordParserStub::new(record);
    AccountLockStateProvider::with_provider_and_parser(
        account_provider,
        delegation_record_parser,
    )
}

#[tokio::test]
async fn test_account_meta_one_properly_locked_writable_and_one_readonly() {
    let (delegated_id, delegation_pda) = delegated_account_ids();
    let lockstate_provider = setup_lockstate_provider(
        vec![
            (delegated_id, account_owned_by_delegation_program()),
            (delegation_pda, account_owned_by_delegation_program()),
        ],
        Some(DelegationRecord::default()),
    );
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
    let (delegated_id, delegation_pda) = delegated_account_ids();
    let writable_id = Pubkey::new_from_array([4u8; 32]);
    let lockstate_provider = setup_lockstate_provider(
        vec![
            (delegated_id, account_owned_by_delegation_program()),
            (delegation_pda, account_owned_by_delegation_program()),
            (writable_id, account_owned_by_system_program()),
        ],
        Some(DelegationRecord::default()),
    );

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
    let (delegated_id, _) = delegated_account_ids();
    let lockstate_provider = setup_lockstate_provider(
        vec![
            (delegated_id, account_owned_by_delegation_program()),
            // Missing delegation account
        ],
        Some(DelegationRecord::default()),
    );
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
async fn test_account_meta_one_locked_writable_with_invalid_delegation_record_and_one_readonly(
) {
    let (delegated_id, delegation_pda) = delegated_account_ids();
    let lockstate_provider = setup_lockstate_provider(
        vec![
            (delegated_id, account_owned_by_delegation_program()),
            (delegation_pda, account_owned_by_delegation_program()),
        ],
        None,
    );
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
    let (delegated_id, delegation_pda) = delegated_account_ids();
    let lockstate_provider = setup_lockstate_provider(
        vec![
            (delegated_id, account_owned_by_delegation_program()),
            (delegation_pda, account_owned_by_delegation_program()),
        ],
        Some(DelegationRecord::default()),
    );
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
    let lockstate_provider =
        setup_lockstate_provider(vec![], Some(DelegationRecord::default()));
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
    let lockstate_provider = setup_lockstate_provider(
        vec![(unlocked_writable_id, account_owned_by_system_program())],
        Some(DelegationRecord::default()),
    );
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
    let lockstate_provider =
        setup_lockstate_provider(vec![], Some(DelegationRecord::default()));
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

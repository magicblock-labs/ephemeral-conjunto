use conjunto_core::DelegationRecord;
use conjunto_lockbox::AccountChainSnapshotProvider;
use conjunto_test_tools::{
    account_provider_stub::AccountProviderStub,
    accounts::{
        account_owned_by_delegation_program, account_owned_by_system_program,
        delegated_account_ids, program_account,
    },
    delegation_record_parser_stub::DelegationRecordParserStub,
    transaction_accounts_holder_stub::TransactionAccountsHolderStub,
};
use conjunto_transwise::{
    endpoint::Endpoint, transaction_account_meta::TransactionAccountMetas,
};
use solana_sdk::{account::Account, pubkey::Pubkey};

fn setup_chain_snapshot_provider(
    accounts: Vec<(Pubkey, Account)>,
    record: Option<DelegationRecord>,
) -> AccountChainSnapshotProvider<AccountProviderStub, DelegationRecordParserStub>
{
    let mut account_provider = AccountProviderStub::default();
    for (pubkey, account) in accounts {
        account_provider.add(pubkey, account);
    }
    let delegation_record_parser = DelegationRecordParserStub::new(record);
    AccountChainSnapshotProvider::with_provider_and_parser(
        account_provider,
        delegation_record_parser,
    )
}

#[tokio::test]
async fn test_account_meta_one_properly_locked_writable_and_one_readonly() {
    let (writable_delegated_id, delegation_pda) = delegated_account_ids();
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (writable_delegated_id, account_owned_by_delegation_program()),
            (delegation_pda, account_owned_by_delegation_program()),
        ],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );
    let readonly_id = Pubkey::new_from_array([4u8; 32]);

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![readonly_id],
        writable: vec![writable_delegated_id],
        payer: Pubkey::new_unique(),
    };

    let account_metas = TransactionAccountMetas::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    let endpoint = Endpoint::from(account_metas);

    eprintln!("{:#?}", endpoint);
    assert!(endpoint.is_ephemeral());
}

#[tokio::test]
async fn test_account_meta_one_properly_delegated_writable_and_one_writable_undelegated(
) {
    let (writable_delegated_id, delegation_pda) = delegated_account_ids();
    let writable_undelegated_id = Pubkey::new_from_array([4u8; 32]);
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (writable_delegated_id, account_owned_by_delegation_program()),
            (delegation_pda, account_owned_by_delegation_program()),
            (writable_undelegated_id, account_owned_by_system_program()),
        ],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![],
        writable: vec![writable_delegated_id, writable_undelegated_id],
        payer: Pubkey::new_unique(),
    };

    let account_metas = TransactionAccountMetas::from_accounts_holder(
        &acc_holder,
        &chain_snapshot_provider,
    )
    .await
    .unwrap();

    let endpoint = Endpoint::from(account_metas);

    eprintln!("{:#?}", endpoint);
    assert!(endpoint.is_unroutable());
}

#[tokio::test]
async fn test_account_meta_one_improperly_locked_writable_and_one_readonly() {
    let (writable_delegated_id, _) = delegated_account_ids();
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (writable_delegated_id, account_owned_by_delegation_program()),
            // Missing delegation account
        ],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );
    let readonly_id = Pubkey::new_from_array([4u8; 32]);

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![readonly_id],
        writable: vec![writable_delegated_id],
        payer: Pubkey::new_unique(),
    };

    let endpoint = Endpoint::from(
        TransactionAccountMetas::from_accounts_holder(
            &acc_holder,
            &chain_snapshot_provider,
        )
        .await
        .unwrap(),
    );

    eprintln!("{:#?}", endpoint);
    assert!(endpoint.is_unroutable());
}

#[tokio::test]
async fn test_account_meta_one_locked_writable_with_invalid_delegation_record_and_one_readonly(
) {
    let (writable_delegated_id, delegation_pda) = delegated_account_ids();
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (writable_delegated_id, account_owned_by_delegation_program()),
            (delegation_pda, account_owned_by_delegation_program()),
        ],
        None, // invalid delegation record for delegated account
    );
    let readonly_id = Pubkey::new_from_array([4u8; 32]);

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![readonly_id],
        writable: vec![writable_delegated_id],
        payer: Pubkey::new_unique(),
    };

    let endpoint = Endpoint::from(
        TransactionAccountMetas::from_accounts_holder(
            &acc_holder,
            &chain_snapshot_provider,
        )
        .await
        .unwrap(),
    );

    eprintln!("{:#?}", endpoint);
    assert!(endpoint.is_unroutable());
}

#[tokio::test]
async fn test_account_meta_one_writable_properly_delegated_and_one_writable_new_account(
) {
    let (writable_delegated_id, delegation_pda) = delegated_account_ids();
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (writable_delegated_id, account_owned_by_delegation_program()),
            (delegation_pda, account_owned_by_delegation_program()),
        ],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );

    let writable_new_account_id = Pubkey::new_from_array([4u8; 32]);

    let acc_holder = TransactionAccountsHolderStub {
        writable: vec![writable_delegated_id, writable_new_account_id],
        ..Default::default()
    };

    let endpoint = Endpoint::from(
        TransactionAccountMetas::from_accounts_holder(
            &acc_holder,
            &chain_snapshot_provider,
        )
        .await
        .unwrap(),
    );

    eprintln!("{:#?}", endpoint);
    assert!(endpoint.is_unroutable());
}

#[tokio::test]
async fn test_account_meta_one_writable_new_account() {
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );

    let writable_new_account_id = Pubkey::new_from_array([4u8; 32]);

    let acc_holder = TransactionAccountsHolderStub {
        writable: vec![writable_new_account_id],
        ..Default::default()
    };

    let endpoint = Endpoint::from(
        TransactionAccountMetas::from_accounts_holder(
            &acc_holder,
            &chain_snapshot_provider,
        )
        .await
        .unwrap(),
    );

    eprintln!("{:#?}", endpoint);
    assert!(endpoint.is_chain());
}

#[tokio::test]
async fn test_account_meta_one_undelegated_writable_that_is_payer() {
    // NOTE: it is very rare to encounter a transaction which would only have
    //       write to one account (same as payer) and we don't expect a
    //       transaction like this to make sense inside the ephemeral validator.
    //       That is the main reason we send it to chain
    let writable_undelegated_id = Pubkey::new_from_array([4u8; 32]);
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![(writable_undelegated_id, account_owned_by_system_program())],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );

    let acc_holder = TransactionAccountsHolderStub {
        writable: vec![writable_undelegated_id],
        payer: writable_undelegated_id,
        ..Default::default()
    };

    let endpoint = Endpoint::from(
        TransactionAccountMetas::from_accounts_holder(
            &acc_holder,
            &chain_snapshot_provider,
        )
        .await
        .unwrap(),
    );

    assert!(endpoint.is_chain());

    let metas = endpoint.into_account_metas();
    assert_eq!(metas.len(), 1);
    assert!(metas[0].is_payer());
}

#[tokio::test]
async fn test_account_meta_one_writable_undelegated_that_is_payer_and_locked_writable(
) {
    let (delegated_id, delegation_pda) = delegated_account_ids();
    let writable_undelegated_id = Pubkey::new_from_array([4u8; 32]);
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (delegated_id, account_owned_by_delegation_program()),
            (delegation_pda, account_owned_by_delegation_program()),
            (writable_undelegated_id, account_owned_by_system_program()),
        ],
        Some(DelegationRecord::default_with_owner(delegated_id)),
    );

    let acc_holder = TransactionAccountsHolderStub {
        writable: vec![writable_undelegated_id, delegated_id],
        payer: writable_undelegated_id,
        ..Default::default()
    };

    let endpoint = Endpoint::from(
        TransactionAccountMetas::from_accounts_holder(
            &acc_holder,
            &chain_snapshot_provider,
        )
        .await
        .unwrap(),
    );

    assert!(endpoint.is_ephemeral());

    let metas = endpoint.into_account_metas();
    assert_eq!(metas.len(), 2);
    assert!(metas[0].is_payer());
}

#[tokio::test]
async fn test_account_meta_one_writable_undelegated_that_is_payer_and_writable_undelegated(
) {
    let writable_undelegated_id = Pubkey::new_from_array([3u8; 32]);
    let writable_undelegated_payer_id = Pubkey::new_from_array([4u8; 32]);
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (writable_undelegated_id, account_owned_by_system_program()),
            (
                writable_undelegated_payer_id,
                account_owned_by_system_program(),
            ),
        ],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![],
        writable: vec![writable_undelegated_payer_id, writable_undelegated_id],
        payer: writable_undelegated_payer_id,
    };

    let endpoint = Endpoint::from(
        TransactionAccountMetas::from_accounts_holder(
            &acc_holder,
            &chain_snapshot_provider,
        )
        .await
        .unwrap(),
    );

    eprintln!("{:#?}", endpoint);
    assert!(endpoint.is_chain());

    let metas = endpoint.into_account_metas();
    assert_eq!(metas.len(), 2);
    assert!(metas[0].is_payer());
}

#[tokio::test]
async fn test_account_meta_one_writable_undelegated_two_readonlys() {
    let writable_undelegated_id = Pubkey::new_from_array([4u8; 32]);
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![(writable_undelegated_id, account_owned_by_system_program())],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );
    let readonly1 = Pubkey::new_from_array([4u8; 32]);
    let readonly2 = Pubkey::new_from_array([5u8; 32]);

    let acc_holder = TransactionAccountsHolderStub {
        writable: vec![writable_undelegated_id],
        readonly: vec![readonly1, readonly2],
        payer: Pubkey::new_unique(),
    };

    let endpoint = Endpoint::from(
        TransactionAccountMetas::from_accounts_holder(
            &acc_holder,
            &chain_snapshot_provider,
        )
        .await
        .unwrap(),
    );

    eprintln!("{:#?}", endpoint);
    assert!(endpoint.is_chain());
}

#[tokio::test]
async fn test_account_meta_two_readonlys() {
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );

    let readonly1 = Pubkey::new_from_array([4u8; 32]);
    let readonly2 = Pubkey::new_from_array([5u8; 32]);

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![readonly1, readonly2],
        ..Default::default()
    };

    let endpoint = Endpoint::from(
        TransactionAccountMetas::from_accounts_holder(
            &acc_holder,
            &chain_snapshot_provider,
        )
        .await
        .unwrap(),
    );

    eprintln!("{:#?}", endpoint);
    assert!(endpoint.is_chain());
}

#[tokio::test]
async fn test_account_meta_two_readonlys_one_program_and_one_writable_undelegated(
) {
    let readonly1 = Pubkey::new_from_array([4u8; 32]);
    let readonly2 = Pubkey::new_from_array([5u8; 32]);
    let writable_undelegated = Pubkey::new_from_array([6u8; 32]);
    let chain_snapshot_provider = setup_chain_snapshot_provider(
        vec![
            (readonly1, account_owned_by_system_program()),
            (readonly2, program_account()),
        ],
        Some(DelegationRecord::default_with_owner(Pubkey::new_unique())),
    );

    let acc_holder = TransactionAccountsHolderStub {
        readonly: vec![readonly1, readonly2],
        writable: vec![writable_undelegated],
        payer: Pubkey::new_unique(),
    };

    let endpoint = Endpoint::from(
        TransactionAccountMetas::from_accounts_holder(
            &acc_holder,
            &chain_snapshot_provider,
        )
        .await
        .unwrap(),
    );
    assert!(endpoint.is_chain());

    let transaction_metas = endpoint.into_account_metas();
    assert_eq!(transaction_metas.len(), 3);
    assert_eq!(*transaction_metas[0].pubkey(), readonly1);
    assert_eq!(*transaction_metas[1].pubkey(), readonly2);
    assert_eq!(*transaction_metas[2].pubkey(), writable_undelegated);
}

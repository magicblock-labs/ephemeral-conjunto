use async_trait::async_trait;
use conjunto_lockbox::{
    account_chain_snapshot::AccountChainSnapshotProvider,
    delegation_record_parser::DelegationRecordParserImpl, RpcProviderConfig,
};
use conjunto_providers::rpc_account_provider::RpcAccountProvider;

use crate::{
    errors::TranswiseResult,
    transaction_accounts_holder::TransactionAccountsHolder,
    transaction_accounts_snapshot::TransactionAccountsSnapshot,
};

#[async_trait]
pub trait AccountFetcher {
    async fn fetch_transaction_accounts_snapshot(
        &self,
        accounts_holder: &TransactionAccountsHolder,
    ) -> TranswiseResult<TransactionAccountsSnapshot>;
}

pub struct RemoteAccountFetcher {
    account_chain_snapshot_provider: AccountChainSnapshotProvider<
        RpcAccountProvider,
        DelegationRecordParserImpl,
    >,
}

impl RemoteAccountFetcher {
    pub fn new(config: RpcProviderConfig) -> Self {
        let account_chain_snapshot_provider = AccountChainSnapshotProvider::new(
            RpcAccountProvider::new(config),
            DelegationRecordParserImpl,
        );
        Self {
            account_chain_snapshot_provider,
        }
    }
}

#[async_trait]
impl AccountFetcher for RemoteAccountFetcher {
    async fn fetch_transaction_accounts_snapshot(
        &self,
        accounts_holder: &TransactionAccountsHolder,
    ) -> TranswiseResult<TransactionAccountsSnapshot> {
        TransactionAccountsSnapshot::from_accounts_holder(
            accounts_holder,
            &self.account_chain_snapshot_provider,
        )
        .await
    }
}

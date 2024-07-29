use conjunto_lockbox::{
    AccountChainSnapshotProvider, DelegationRecordParserImpl,
};
use conjunto_providers::{
    rpc_account_provider::RpcAccountProvider,
    rpc_provider_config::RpcProviderConfig,
};
use solana_sdk::transaction::{SanitizedTransaction, VersionedTransaction};

use crate::{
    endpoint::Endpoint, errors::TranswiseResult,
    transaction_account_meta::TransactionAccountMetas,
    transaction_accounts_holder::TransactionAccountsHolder,
};

/// The API that allows us to guide a transaction given a cluster
/// Guiding decisions are made by consulting the state of accounts on chain
/// See [../examples/guiding_transactions.rs] for more info.
pub struct Transwise {
    account_chain_snapshot_provider: AccountChainSnapshotProvider<
        RpcAccountProvider,
        DelegationRecordParserImpl,
    >,
}

impl Transwise {
    pub fn new(config: RpcProviderConfig) -> Self {
        let account_chain_snapshot_provider = AccountChainSnapshotProvider::<
            RpcAccountProvider,
            DelegationRecordParserImpl,
        >::new(config);
        Self {
            account_chain_snapshot_provider,
        }
    }

    /// Extracts information of all accounts involved in the transaction and
    /// checks their lock state on chain.
    /// This method is a convenience API but inefficient since it validates
    /// all accounts found inside the transaction without us being able to omit
    /// checks for some of them
    pub async fn account_metas_from_versioned_transaction(
        &self,
        tx: &VersionedTransaction,
    ) -> TranswiseResult<TransactionAccountMetas> {
        TransactionAccountMetas::from_versioned_transaction(
            tx,
            &self.account_chain_snapshot_provider,
        )
        .await
    }

    /// Extracts information of all accounts involved in the transaction and
    /// checks their lock state on chain.
    /// This method is a convenience API but inefficient since it validates
    /// all accounts found inside the transaction without us being able to omit
    /// checks for some of them
    pub async fn account_metas_from_sanitized_transaction(
        &self,
        tx: &SanitizedTransaction,
    ) -> TranswiseResult<TransactionAccountMetas> {
        TransactionAccountMetas::from_sanitized_transaction(
            tx,
            &self.account_chain_snapshot_provider,
        )
        .await
    }

    /// Extracts information of all provided accounts and checks their lock state on chain.
    /// This method allows providing exacty the transaction accounts that we need checked
    /// and thus is preferred due to the lower overhead.
    pub async fn account_metas(
        &self,
        accounts: &TransactionAccountsHolder,
    ) -> TranswiseResult<TransactionAccountMetas> {
        TransactionAccountMetas::from_accounts_holder(
            accounts,
            &self.account_chain_snapshot_provider,
        )
        .await
    }

    /// Extracts information of all accounts involved in the transaction,
    /// checks their lock state on chain and based on that returns an endpoint.
    pub async fn guide_versioned_transaction(
        &self,
        tx: &VersionedTransaction,
    ) -> TranswiseResult<Endpoint> {
        Ok(Endpoint::from(
            self.account_metas_from_versioned_transaction(tx).await?,
        ))
    }

    /// Extracts information of all accounts involved in the transaction,
    /// checks their lock state on chain and based on that returns an endpoint.
    pub async fn guide_sanitized_transaction(
        &self,
        tx: &SanitizedTransaction,
    ) -> TranswiseResult<Endpoint> {
        Ok(Endpoint::from(
            self.account_metas_from_sanitized_transaction(tx).await?,
        ))
    }
}

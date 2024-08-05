use conjunto_providers::rpc_provider_config::RpcProviderConfig;
use solana_sdk::transaction::{SanitizedTransaction, VersionedTransaction};

use crate::{
    account_fetcher::{AccountFetcher, RemoteAccountFetcher},
    endpoint::Endpoint,
    errors::TranswiseResult,
    transaction_accounts_holder::TransactionAccountsHolder,
    transaction_accounts_snapshot::TransactionAccountsSnapshot,
};

/// The API that allows us to guide a transaction given a cluster
/// Guiding decisions are made by consulting the state of accounts on chain
/// See [../examples/guiding_transactions.rs] for more info.
pub struct Transwise {
    account_fetcher: RemoteAccountFetcher,
}

impl Transwise {
    pub fn new(config: RpcProviderConfig) -> Self {
        Self {
            account_fetcher: RemoteAccountFetcher::new(config),
        }
    }

    /// Extracts information of all accounts involved in the transaction,
    /// checks their lock state on chain and based on that returns an endpoint.
    pub async fn guide_versioned_transaction(
        &self,
        tx: &VersionedTransaction,
    ) -> TranswiseResult<Endpoint> {
        Ok(Endpoint::from(
            self.transaction_accounts_snapshot_from_versioned_transaction(tx)
                .await?,
        ))
    }

    /// Extracts information of all accounts involved in the transaction,
    /// checks their lock state on chain and based on that returns an endpoint.
    pub async fn guide_sanitized_transaction(
        &self,
        tx: &SanitizedTransaction,
    ) -> TranswiseResult<Endpoint> {
        Ok(Endpoint::from(
            self.transaction_accounts_snapshot_from_sanitized_transaction(tx)
                .await?,
        ))
    }

    /// Extracts information of all accounts involved in the transaction and
    /// checks their lock state on chain.
    /// This method is a convenience API but inefficient since it validates
    /// all accounts found inside the transaction without us being able to omit
    /// checks for some of them
    async fn transaction_accounts_snapshot_from_versioned_transaction(
        &self,
        tx: &VersionedTransaction,
    ) -> TranswiseResult<TransactionAccountsSnapshot> {
        self.account_fetcher
            .fetch_transaction_accounts_snapshot(
                &TransactionAccountsHolder::try_from(tx)?,
            )
            .await
    }

    /// Extracts information of all accounts involved in the transaction and
    /// checks their lock state on chain.
    /// This method is a convenience API but inefficient since it validates
    /// all accounts found inside the transaction without us being able to omit
    /// checks for some of them
    async fn transaction_accounts_snapshot_from_sanitized_transaction(
        &self,
        tx: &SanitizedTransaction,
    ) -> TranswiseResult<TransactionAccountsSnapshot> {
        self.account_fetcher
            .fetch_transaction_accounts_snapshot(
                &TransactionAccountsHolder::try_from(tx)?,
            )
            .await
    }
}

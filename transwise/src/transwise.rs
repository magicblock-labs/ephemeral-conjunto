use conjunto_lockbox::{
    account_chain_snapshot_provider::AccountChainSnapshotProvider,
    delegation_record_parser_impl::DelegationRecordParserImpl,
};
use conjunto_providers::{
    rpc_account_provider::RpcAccountProvider,
    rpc_provider_config::RpcProviderConfig,
};
use solana_sdk::transaction::{SanitizedTransaction, VersionedTransaction};

use crate::{
    endpoint::Endpoint, errors::TranswiseResult,
    transaction_accounts_holder::TransactionAccountsHolder,
    transaction_accounts_snapshot::TransactionAccountsSnapshot,
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
        let account_chain_snapshot_provider = AccountChainSnapshotProvider::new(
            RpcAccountProvider::new(config),
            DelegationRecordParserImpl,
        );
        Self {
            account_chain_snapshot_provider,
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
        TransactionAccountsSnapshot::from_accounts_holder(
            &TransactionAccountsHolder::try_from(tx)?,
            &self.account_chain_snapshot_provider,
            None,
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
        TransactionAccountsSnapshot::from_accounts_holder(
            &TransactionAccountsHolder::try_from(tx)?,
            &self.account_chain_snapshot_provider,
            None,
        )
        .await
    }
}

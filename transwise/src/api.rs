use conjunto_lockbox::AccountLockStateProvider;
use conjunto_providers::{
    rpc_account_provider::RpcAccountProvider,
    rpc_provider_config::RpcProviderConfig,
};
use solana_sdk::transaction::{SanitizedTransaction, VersionedTransaction};

use crate::{
    errors::TranswiseResult,
    trans_account_meta::{Endpoint, TransAccountMetas},
    validated_accounts::{ValidateAccountsConfig, ValidatedAccounts},
};

/// The API that allows us to guide a transaction given a cluster
/// Guiding decisions are made by consulting the state of accounts on chain
/// See [../examples/guiding_transactions.rs] for more info.
pub struct Transwise {
    account_lock_state_provider: AccountLockStateProvider<RpcAccountProvider>,
}

impl Transwise {
    pub fn new(config: RpcProviderConfig) -> Self {
        let account_lock_state_provider =
            AccountLockStateProvider::<RpcAccountProvider>::new(config);
        Self {
            account_lock_state_provider,
        }
    }

    /// Extracts information of all accounts involved in the transaction and
    /// checks their lock state on chain.
    pub async fn account_metas_from_versioned_transaction(
        &self,
        tx: &VersionedTransaction,
    ) -> TranswiseResult<TransAccountMetas> {
        TransAccountMetas::from_versioned_transaction(
            tx,
            &self.account_lock_state_provider,
        )
        .await
    }

    /// Extracts information of all accounts involved in the transaction and
    /// checks their lock state on chain.
    pub async fn account_metas_from_sanitized_transaction(
        &self,
        tx: &SanitizedTransaction,
    ) -> TranswiseResult<TransAccountMetas> {
        TransAccountMetas::from_sanitized_transaction(
            tx,
            &self.account_lock_state_provider,
        )
        .await
    }

    /// Extracts information of all accounts involved in the transaction,
    /// checks their lock state on chain and based on that returns an endpoint.
    pub async fn guide_versioned_transaction(
        &self,
        tx: &VersionedTransaction,
    ) -> TranswiseResult<Endpoint> {
        Ok(self
            .account_metas_from_versioned_transaction(tx)
            .await?
            .into_endpoint())
    }

    /// Extracts information of all accounts involved in the transaction,
    /// checks their lock state on chain and based on that returns an endpoint.
    pub async fn guide_sanitized_transaction(
        &self,
        tx: &SanitizedTransaction,
    ) -> TranswiseResult<Endpoint> {
        Ok(self
            .account_metas_from_sanitized_transaction(tx)
            .await?
            .into_endpoint())
    }

    /// Extracts information of all accounts involved in the transaction, validates
    /// them and returns the result containing writable and readonly accounts.
    /// The checks make sure that all writable accounts are either locked or conform
    /// to what's specified in the config.
    pub async fn validated_accounts_from_versioned_transaction(
        &self,
        tx: &VersionedTransaction,
        config: &ValidateAccountsConfig,
    ) -> TranswiseResult<ValidatedAccounts> {
        let account_metas =
            self.account_metas_from_versioned_transaction(tx).await?;
        (&account_metas, config).try_into()
    }

    /// Extracts information of all accounts involved in the transaction, validates
    /// them and returns the result containing writable and readonly accounts.
    /// The checks make sure that all writable accounts are either locked or conform
    /// to what's specified in the config.
    pub async fn validated_accounts_from_sanitized_transaction(
        &self,
        tx: &SanitizedTransaction,
        config: &ValidateAccountsConfig,
    ) -> TranswiseResult<ValidatedAccounts> {
        let account_metas =
            self.account_metas_from_sanitized_transaction(tx).await?;
        (&account_metas, config).try_into()
    }
}

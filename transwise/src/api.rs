use async_trait::async_trait;
use conjunto_lockbox::{AccountLockStateProvider, DelegationRecordParserImpl};
use conjunto_providers::{
    rpc_account_provider::RpcAccountProvider,
    rpc_provider_config::RpcProviderConfig,
};
use solana_sdk::transaction::{SanitizedTransaction, VersionedTransaction};

use crate::{
    errors::TranswiseResult,
    trans_account_meta::{
        Endpoint, TransAccountMetas, TransactionAccountsHolder,
    },
    validated_accounts::{ValidateAccountsConfig, ValidatedAccounts},
};

#[async_trait]
pub trait ValidatedAccountsProvider {
    /// Extracts information of all accounts involved in the transaction, validates
    /// them and returns the result containing writable and readonly accounts.
    /// The checks make sure that all writable accounts are either locked or conform
    /// to what's specified in the config.
    /// It is inefficent since it does not allow us to omit checks for some accounts.
    /// Therefore the [Self::validate_accounts] method is preferred.
    async fn validated_accounts_from_versioned_transaction(
        &self,
        tx: &VersionedTransaction,
        config: &ValidateAccountsConfig,
    ) -> TranswiseResult<ValidatedAccounts>;

    /// Extracts information of all accounts involved in the transaction, validates
    /// them and returns the result containing writable and readonly accounts.
    /// The checks make sure that all writable accounts are either locked or conform
    /// to what's specified in the config.
    /// It is inefficent since it does not allow us to omit checks for some accounts.
    /// Therefore the [Self::validate_accounts] method is preferred.
    async fn validated_accounts_from_sanitized_transaction(
        &self,
        tx: &SanitizedTransaction,
        config: &ValidateAccountsConfig,
    ) -> TranswiseResult<ValidatedAccounts>;

    /// Extracts information of the provided accounts, validates
    /// them and returns the result containing writable and readonly accounts.
    /// The checks make sure that all writable accounts are either locked or conform
    /// to what's specified in the config.
    async fn validate_accounts(
        &self,
        transaction_accounts: &TransactionAccountsHolder,
        config: &ValidateAccountsConfig,
    ) -> TranswiseResult<ValidatedAccounts>;
}

pub trait TransactionAccountsExtractor {
    fn accounts_from_versioned_transaction(
        &self,
        tx: &VersionedTransaction,
    ) -> TransactionAccountsHolder;

    fn accounts_from_sanitized_transaction(
        &self,
        tx: &SanitizedTransaction,
    ) -> TransactionAccountsHolder;
}

/// The API that allows us to guide a transaction given a cluster
/// Guiding decisions are made by consulting the state of accounts on chain
/// See [../examples/guiding_transactions.rs] for more info.
pub struct Transwise {
    account_lock_state_provider: AccountLockStateProvider<
        RpcAccountProvider,
        DelegationRecordParserImpl,
    >,
}

impl Transwise {
    pub fn new(config: RpcProviderConfig) -> Self {
        let account_lock_state_provider = AccountLockStateProvider::<
            RpcAccountProvider,
            DelegationRecordParserImpl,
        >::new(config);
        Self {
            account_lock_state_provider,
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
    ) -> TranswiseResult<TransAccountMetas> {
        TransAccountMetas::from_versioned_transaction(
            tx,
            &self.account_lock_state_provider,
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
    ) -> TranswiseResult<TransAccountMetas> {
        TransAccountMetas::from_sanitized_transaction(
            tx,
            &self.account_lock_state_provider,
        )
        .await
    }

    /// Extracts information of all provided accounts and checks their lock state on chain.
    /// This method allows providing exacty the transaction accounts that we need checked
    /// and thus is preferred due to the lower overhead.
    pub async fn account_metas(
        &self,
        accounts: &TransactionAccountsHolder,
    ) -> TranswiseResult<TransAccountMetas> {
        TransAccountMetas::from_accounts_holder(
            accounts,
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
}

#[async_trait]
impl ValidatedAccountsProvider for Transwise {
    async fn validated_accounts_from_versioned_transaction(
        &self,
        tx: &VersionedTransaction,
        config: &ValidateAccountsConfig,
    ) -> TranswiseResult<ValidatedAccounts> {
        let account_metas =
            self.account_metas_from_versioned_transaction(tx).await?;
        (&account_metas, config).try_into()
    }

    async fn validated_accounts_from_sanitized_transaction(
        &self,
        tx: &SanitizedTransaction,
        config: &ValidateAccountsConfig,
    ) -> TranswiseResult<ValidatedAccounts> {
        let account_metas =
            self.account_metas_from_sanitized_transaction(tx).await?;
        (&account_metas, config).try_into()
    }

    async fn validate_accounts(
        &self,
        transaction_accounts: &TransactionAccountsHolder,
        config: &ValidateAccountsConfig,
    ) -> TranswiseResult<ValidatedAccounts> {
        let account_metas = self.account_metas(transaction_accounts).await?;
        (&account_metas, config).try_into()
    }
}

impl TransactionAccountsExtractor for Transwise {
    fn accounts_from_versioned_transaction(
        &self,
        tx: &VersionedTransaction,
    ) -> TransactionAccountsHolder {
        TransactionAccountsHolder::from(tx)
    }

    fn accounts_from_sanitized_transaction(
        &self,
        tx: &SanitizedTransaction,
    ) -> TransactionAccountsHolder {
        TransactionAccountsHolder::from(tx)
    }
}

use conjunto_lockbox::AccountLockStateProvider;
use conjunto_providers::{
    rpc_account_provider::RpcAccountProvider,
    rpc_provider_config::RpcProviderConfig,
};
use solana_sdk::transaction::{SanitizedTransaction, VersionedTransaction};

use crate::{
    errors::TranswiseResult,
    trans_account_meta::{Endpoint, TransAccountMetas},
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

    pub async fn guide_versioned_transaction(
        &self,
        tx: &VersionedTransaction,
    ) -> TranswiseResult<Endpoint> {
        let account_metas = TransAccountMetas::from_versioned_transaction(
            tx,
            &self.account_lock_state_provider,
        )
        .await?;
        Ok(account_metas.into_endpoint())
    }

    pub async fn guide_sanitized_transaction(
        &self,
        tx: &SanitizedTransaction,
    ) -> TranswiseResult<Endpoint> {
        let account_metas = TransAccountMetas::from_sanitized_transaction(
            tx,
            &self.account_lock_state_provider,
        )
        .await?;
        Ok(account_metas.into_endpoint())
    }
}

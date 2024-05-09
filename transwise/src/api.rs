use conjunto_lockbox::{
    accounts::{RpcAccountProvider, RpcAccountProviderConfig},
    AccountLockStateProvider,
};
use solana_sdk::transaction::SanitizedTransaction;

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
    pub fn new(config: RpcAccountProviderConfig) -> Self {
        let account_lock_state_provider =
            AccountLockStateProvider::<RpcAccountProvider>::new(config);
        Self {
            account_lock_state_provider,
        }
    }

    pub async fn guide_transaction(
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

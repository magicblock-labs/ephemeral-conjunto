use async_trait::async_trait;
use solana_sdk::{account::Account, pubkey::Pubkey};

use crate::errors::LockboxResult;
pub(crate) mod predicates;
pub(crate) mod rpc_account_provider;

pub use rpc_account_provider::{RpcAccountProvider, RpcAccountProviderConfig};

#[async_trait]
pub trait AccountProvider {
    async fn get_account(
        &self,
        pubkey: &Pubkey,
    ) -> LockboxResult<Option<Account>>;
    async fn get_multiple_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> LockboxResult<Vec<Option<Account>>>;
}

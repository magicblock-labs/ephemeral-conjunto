use async_trait::async_trait;
use solana_sdk::{account::Account, pubkey::Pubkey};

use crate::errors::CoreResult;

#[async_trait]
pub trait AccountProvider {
    async fn get_account(&self, pubkey: &Pubkey)
        -> CoreResult<Option<Account>>;
    async fn get_multiple_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> CoreResult<Vec<Option<Account>>>;
}

pub trait TransactionAccountsHolder {
    fn get_writable(&self) -> Vec<Pubkey>;
    fn get_readonly(&self) -> Vec<Pubkey>;
}

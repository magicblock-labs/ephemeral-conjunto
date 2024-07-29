use std::collections::HashMap;

use async_trait::async_trait;
use conjunto_core::{errors::CoreResult, AccountProvider};
use solana_sdk::{account::Account, clock::Slot, pubkey::Pubkey};

#[derive(Default)]
pub struct AccountProviderStub {
    pub accounts: HashMap<Pubkey, Account>,
}

impl AccountProviderStub {
    pub fn add(&mut self, pubkey: Pubkey, account: Account) {
        self.accounts.insert(pubkey, account);
    }
}

#[async_trait]
impl AccountProvider for AccountProviderStub {
    async fn get_account(
        &self,
        pubkey: &Pubkey,
    ) -> CoreResult<(Slot, Option<Account>)> {
        Ok((0, self.accounts.get(pubkey).cloned()))
    }

    async fn get_multiple_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> CoreResult<(Slot, Vec<Option<Account>>)> {
        Ok((
            0,
            pubkeys
                .iter()
                .map(|pubkey| self.accounts.get(pubkey).cloned())
                .collect(),
        ))
    }
}

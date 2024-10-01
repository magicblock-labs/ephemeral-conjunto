use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use conjunto_core::{errors::CoreResult, AccountProvider};
use solana_sdk::{account::Account, clock::Slot, pubkey::Pubkey};

#[derive(Default)]
pub struct AccountProviderStub {
    pub at_slot: Slot,
    pub accounts: Arc<RwLock<HashMap<Pubkey, Account>>>,
}

impl AccountProviderStub {
    pub fn add(&mut self, pubkey: Pubkey, account: Account) {
        self.accounts.write().unwrap().insert(pubkey, account);
    }
    fn get(&self, pubkey: &Pubkey) -> Option<Account> {
        self.accounts.read().unwrap().get(pubkey).cloned()
    }
}

#[async_trait]
impl AccountProvider for AccountProviderStub {
    async fn get_account(
        &self,
        pubkey: &Pubkey,
    ) -> CoreResult<(Slot, Option<Account>)> {
        Ok((self.at_slot, self.get(pubkey)))
    }

    async fn get_multiple_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> CoreResult<(Slot, Vec<Option<Account>>)> {
        Ok((
            self.at_slot,
            pubkeys.iter().map(|pubkey| self.get(pubkey)).collect(),
        ))
    }
}

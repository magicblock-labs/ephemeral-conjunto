use async_trait::async_trait;
use solana_sdk::{
    account::Account, pubkey::Pubkey, signature::Signature, transaction,
};

use crate::{errors::CoreResult, DelegationRecord};

#[async_trait]
pub trait AccountProvider:
    std::marker::Sync + std::marker::Send + 'static
{
    async fn get_account(&self, pubkey: &Pubkey)
        -> CoreResult<Option<Account>>;
    async fn get_multiple_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> CoreResult<Vec<Option<Account>>>;
}

pub trait AccountsHolder {
    fn get_writable(&self) -> Vec<Pubkey>;
    fn get_readonly(&self) -> Vec<Pubkey>;
}

#[async_trait]
pub trait SignatureStatusProvider:
    std::marker::Sync + std::marker::Send + 'static
{
    async fn get_signature_status(
        &self,
        signature: &Signature,
    ) -> CoreResult<Option<transaction::Result<()>>>;
}

pub trait DelegationRecordParser {
    fn try_parse(&self, data: &[u8]) -> CoreResult<DelegationRecord>;
}

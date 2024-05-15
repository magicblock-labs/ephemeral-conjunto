use conjunto_core::AccountsHolder;
use solana_sdk::pubkey::Pubkey;

#[derive(Default)]
pub struct TransactionAccountsHolderStub {
    pub readonly: Vec<Pubkey>,
    pub writable: Vec<Pubkey>,
}
impl AccountsHolder for TransactionAccountsHolderStub {
    fn get_writable(&self) -> Vec<Pubkey> {
        self.writable.clone()
    }
    fn get_readonly(&self) -> Vec<Pubkey> {
        self.readonly.clone()
    }
}

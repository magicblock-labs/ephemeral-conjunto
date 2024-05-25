use solana_sdk::pubkey::Pubkey;
use thiserror::Error;

pub type TranswiseResult<T> = std::result::Result<T, TranswiseError>;

#[derive(Error, Debug)]
pub enum TranswiseError {
    #[error("LockboxError")]
    LockboxError(#[from] conjunto_lockbox::errors::LockboxError),

    #[error("CoreError")]
    CoreError(#[from] conjunto_core::errors::CoreError),

    #[error("Not all writable accounts are locked")]
    NotAllWritablesLocked {
        locked: Vec<Pubkey>,
        unlocked: Vec<Pubkey>,
    },

    #[error("Writables inconsistent accounts")]
    WritablesIncludeInconsistentAccounts { inconsistent: Vec<Pubkey> },

    #[error("Writables include new accounts")]
    WritablesIncludeNewAccounts { new_accounts: Vec<Pubkey> },

    #[error("Transaction is missing payer account")]
    TransactionIsMissingPayerAccount,
}

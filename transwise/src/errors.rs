use solana_sdk::pubkey::Pubkey;
use thiserror::Error;

pub type TranswiseResult<T> = std::result::Result<T, TranswiseError>;

#[derive(Error, Debug)]
pub enum TranswiseError {
    #[error("LockboxError")]
    LockboxError(#[from] conjunto_lockbox::errors::LockboxError),

    #[error("CoreError")]
    CoreError(#[from] conjunto_core::errors::CoreError),

    #[error("Not all writable accounts are delegated")]
    NotAllWritablesDelegated {
        writable_delegated_pubkeys: Vec<Pubkey>,
        writable_undelegated_non_payer_pubkeys: Vec<Pubkey>,
    },

    #[error("Writables inconsistent accounts")]
    WritablesIncludeInconsistentAccounts {
        writable_inconsistent_pubkeys: Vec<Pubkey>,
    },

    #[error("Writables include new accounts")]
    WritablesIncludeNewAccounts { writable_new_pubkeys: Vec<Pubkey> },

    #[error("Transaction is missing payer account")]
    TransactionIsMissingPayerAccount,

    #[error("ValidateAccountsConfig is configured improperly")]
    ValidateAccountsConfigIsInvalid(String),
}

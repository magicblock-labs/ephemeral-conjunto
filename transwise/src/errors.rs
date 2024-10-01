use solana_sdk::pubkey::Pubkey;
use thiserror::Error;

pub type TranswiseResult<T> = std::result::Result<T, TranswiseError>;

#[derive(Error, Debug)]
pub enum TranswiseError {
    #[error("LockboxError")]
    LockboxError(#[from] conjunto_lockbox::errors::LockboxError),

    #[error("CoreError")]
    CoreError(#[from] conjunto_core::errors::CoreError),

    #[error("Transaction includes non-delegated account(s) as writables")]
    TransactionIncludeUndelegatedAccountsAsWritable {
        writable_undelegated_pubkeys: Vec<Pubkey>,
    },

    #[error("Transaction is missing payer account")]
    TransactionIsMissingPayerAccount,

    #[error("ValidateAccountsConfig is configured improperly")]
    ValidateAccountsConfigIsInvalid(String),
}

use solana_sdk::{account::Account, pubkey::Pubkey};
use thiserror::Error;

pub type LockboxResult<T> = std::result::Result<T, LockboxError>;

#[derive(Error, Debug)]
pub enum LockboxError {
    #[error("RpcClientError")]
    RpcClientError(#[from] solana_rpc_client_api::client_error::Error),
    #[error("ConjuntoCoreError")]
    ConjuntoCoreError(#[from] conjunto_core::errors::CoreError),
    #[error("InvalidFetch")]
    InvalidFetch {
        fetched_pubkeys: Vec<Pubkey>,
        fetched_accounts: Vec<Option<Account>>,
    },
    #[error("Failed to parse account data")]
    FailedToParseDelegationRecord(String),
}

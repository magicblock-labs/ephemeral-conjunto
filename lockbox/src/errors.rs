use thiserror::Error;

pub type LockboxResult<T> = std::result::Result<T, LockboxError>;

#[derive(Error, Debug)]
pub enum LockboxError {
    #[error("RpcClientError")]
    RpcClientError(#[from] solana_rpc_client_api::client_error::Error),
    #[error("ConjuntoCoreError")]
    ConjuntoCoreError(#[from] conjunto_core::errors::CoreError),
    #[error("Failed to get account from cluster")]
    FailedToGetAccountFromCluster,
}
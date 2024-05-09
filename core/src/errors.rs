use thiserror::Error;

pub type CoreResult<T> = std::result::Result<T, CoreError>;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("RpcClientError")]
    RpcClientError(#[from] solana_rpc_client_api::client_error::Error),
    #[error("Failed to get account from cluster")]
    FailedToGetAccountFromCluster,
}

use thiserror::Error;

pub type DirectorRpcResult<T> = Result<T, DirectorRpcError>;

#[derive(Debug, Error)]
pub enum DirectorRpcError {
    #[error("JsonRpcRegisterMethodError")]
    JsonRpcRegisterMethodError(#[from] jsonrpsee::core::RegisterMethodError),
    #[error("JsonRpcClientError")]
    JsonRpcClientError(#[from] jsonrpsee::core::client::Error),
    #[error("StdIoError")]
    StdIoError(#[from] std::io::Error),
}

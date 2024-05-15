use jsonrpsee::types::{ErrorCode, ErrorObject, ErrorObjectOwned};
use serde::Serialize;

pub fn invalid_params(msg: String) -> ErrorObjectOwned {
    ErrorObject::owned(ErrorCode::InvalidParams.code(), msg, None::<String>)
}

#[derive(Debug)]
pub enum ServerErrorCode {
    FailedToFetchEndpointInformation = 0,
    TransactionUnroutable = 1,
    RpcClientError = 2,
}

pub fn server_error(msg: String, code: ServerErrorCode) -> ErrorObjectOwned {
    ErrorObject::owned(code as i32, msg, None::<String>)
}

pub fn server_error_with_data<S: Serialize>(
    msg: String,
    code: ServerErrorCode,
    data: S,
) -> ErrorObjectOwned {
    ErrorObject::owned(code as i32, msg, Some(data))
}

use thiserror::Error;

pub type TranswiseResult<T> = std::result::Result<T, TranswiseError>;

#[derive(Error, Debug)]
pub enum TranswiseError {
    #[error("LockboxError")]
    LockboxError(#[from] conjunto_lockbox::errors::LockboxError),
}

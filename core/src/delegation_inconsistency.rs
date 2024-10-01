use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DelegationInconsistency {
    AccountInvalidOwner,
    DelegationRecordNotFound,
    DelegationRecordInvalidOwner,
    DelegationRecordDataInvalid(String),
}

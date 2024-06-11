use conjunto_core::{CommitFrequency, DelegationRecord};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct LockConfig {
    /// The frequency at which the account should be committed to chain
    pub commit_frequency: CommitFrequency,
    /// The current owner of delegated accounts is the delegation
    /// program.
    /// Here we include the original owner of the account before delegation.
    /// This info is provided via the delegation record.
    pub owner: Pubkey,
}

impl From<DelegationRecord> for LockConfig {
    fn from(record: DelegationRecord) -> Self {
        Self {
            commit_frequency: record.commit_frequency,
            owner: record.owner,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LockInconsistency {
    DelegationAccountNotFound,
    BufferAccountInvalidOwner,
    DelegationAccountInvalidOwner,
    DelegationRecordAccountDataInvalid(String),
}

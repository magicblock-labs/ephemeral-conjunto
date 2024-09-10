use std::{fmt, time::Duration};

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum CommitFrequency {
    /// Commit every time after n number of milliseconds passed.
    Millis(u64),
}

impl Default for CommitFrequency {
    fn default() -> Self {
        CommitFrequency::Millis(300_000)
    }
}

impl fmt::Display for CommitFrequency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommitFrequency::Millis(millis) => write!(f, "{}ms", millis),
        }
    }
}

impl From<CommitFrequency> for Duration {
    fn from(freq: CommitFrequency) -> Duration {
        match freq {
            CommitFrequency::Millis(millis) => Duration::from_millis(millis),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct DelegationRecord {
    /// The specified delegation authority
    pub authority: Pubkey,
    /// The original owner of the account
    pub owner: Pubkey,
    /// The slot at which the delegation was created
    pub delegation_slot: u64,
    /// The frequency at which to commit the account state of the ephemeral validator back to the chain.
    pub commit_frequency: CommitFrequency,
}

impl DelegationRecord {
    pub fn default_with_owner(owner: Pubkey) -> Self {
        Self {
            authority: Pubkey::default(),
            owner,
            delegation_slot: 0,
            commit_frequency: CommitFrequency::Millis(1_000),
        }
    }
}

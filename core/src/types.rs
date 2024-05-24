use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

// -----------------
// GuideStrategy
// -----------------
#[derive(Debug, PartialEq, Eq)]
pub enum GuideStrategy {
    /// Forward to chain
    Chain,
    /// Forward to ephemeral
    Ephemeral,
    /// Forward to both chain and ephemeral
    Both,
    /// Forward to ephemeral if that validator has the account of given address,
    /// otherwise forward to chain
    /// - *param.0*: address
    /// - *param.1*: is_subscription
    TryEphemeralForAccount(String, bool),
    /// Forward to ephemeral if that validator has the program of given address,
    /// otherwise forward to chain
    /// - *param.0*: program_id
    /// - *param.1*: is_subscription
    TryEphemeralForProgram(String, bool),
    /// Forward to ephemeral if that validator has the transaction signature,
    /// otherwise forward to both for subscriptions since the transaction may come
    /// in after the request.
    /// For single requests forward to ephemeral if the signature is found, otherwise
    /// to chain
    /// - *param.0*: signature
    /// - *param.1*: is_subscription
    TryEphemeralForSignature(String, bool),
}

// -----------------
// RequestEndpoint
// -----------------
#[derive(Debug, PartialEq, Eq)]
pub enum RequestEndpoint {
    /// Forward to chain only
    Chain,
    /// Forward to ephemeral only
    Ephemeral,
    /// Forward to both chain and ephemeral
    Both,
}

// -----------------
// DelegationRecord
// -----------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum CommitFrequency {
    /// Commit every time after n number of milliseconds passed.
    Millis(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DelegationRecord {
    /// The original owner of the account
    pub owner: Pubkey,
    /// The frequency at which to commit the account state of the ephemeral
    /// validator to the chain.
    pub commit_frequency: CommitFrequency,
}

impl DelegationRecord {
    pub fn default_with_owner(owner: Pubkey) -> Self {
        Self {
            owner,
            commit_frequency: CommitFrequency::Millis(1_000),
        }
    }
}

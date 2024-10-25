use conjunto_core::{
    delegation_inconsistency::DelegationInconsistency,
    delegation_record::DelegationRecord,
};
use serde::{Deserialize, Serialize};
use solana_sdk::{account::Account, pubkey::Pubkey};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum AccountChainState {
    /// The feepayer account is an account that has no data (optionally lamports)
    /// - It can be used as a feepayer in the ephemeral validator
    /// - It should never be allocated in the ephemeral validator
    /// - It's owned by the system program
    /// - It's on curve
    /// - It can only be used for paying fees!
    /// - Its lamport balance must be escrowed to exist in the ephemeral validator
    FeePayer { lamports: u64, owner: Pubkey },
    /// The account is not delegated and contains arbitrary data
    /// - It should never be used as writable in the ephemeral validator
    /// - It can be used as a readonly in the ephemeral validator
    Undelegated {
        account: Account,
        delegation_inconsistency: DelegationInconsistency,
    },
    /// The account was found on chain in a proper delegated state which means we
    /// also found the related accounts like the buffer and delegation
    /// - It can be written to inside of the ephemeral validator
    Delegated {
        account: Account,
        delegation_record: DelegationRecord,
    },
}

impl AccountChainState {
    pub fn is_feepayer(&self) -> bool {
        matches!(self, AccountChainState::FeePayer { .. })
    }
    pub fn is_undelegated(&self) -> bool {
        matches!(self, AccountChainState::Undelegated { .. })
    }
    pub fn is_delegated(&self) -> bool {
        matches!(self, AccountChainState::Delegated { .. })
    }
    pub fn account(&self) -> Option<&Account> {
        match self {
            AccountChainState::FeePayer { .. } => None,
            AccountChainState::Undelegated { account, .. } => Some(account),
            AccountChainState::Delegated { account, .. } => Some(account),
        }
    }
}

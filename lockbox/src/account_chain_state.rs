use conjunto_core::{
    delegation_inconsistency::DelegationInconsistency,
    delegation_record::DelegationRecord,
};
use serde::{Deserialize, Serialize};
use solana_sdk::{account::Account, pubkey::Pubkey};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum AccountChainState {
    /// The account is not present on chain and thus not delegated either
    /// In this case we assume that this is an account that temporarily exists
    /// on the ephemeral validator and will not have to be undelegated.
    /// However in the short term we don't allow new accounts to be created inside
    /// the validator which means that we reject any transactions that attempt to do so
    NewAccount,
    /// The account was found on chain and is not delegated and therefore should
    /// not be used as writable on the ephemeral validator unless otherwise allowed
    /// via the `require_delegation=false` setting.
    Undelegated { account: Account },
    /// The account was found on chain in a proper delegated state which means we
    /// also found the related accounts like the buffer and delegation
    /// NOTE: commit records and state diff accountsk are not checked since an
    /// account is delegated and then used before the validator commits a state change.
    Delegated {
        account: Account,
        delegation_pda: Pubkey,
        delegation_record: DelegationRecord,
    },
    /// The account was found on chain and was partially delegated which means that
    /// it is owned by the delegation program but one or more of the related
    /// accounts were either not present or not owned by the delegation program
    Inconsistent {
        account: Account,
        delegation_pda: Pubkey,
        delegation_inconsistencies: Vec<DelegationInconsistency>,
    },
}

impl AccountChainState {
    pub fn is_new(&self) -> bool {
        matches!(self, AccountChainState::NewAccount)
    }

    pub fn is_delegated(&self) -> bool {
        matches!(self, AccountChainState::Delegated { .. })
    }

    pub fn is_undelegated(&self) -> bool {
        matches!(self, AccountChainState::Undelegated { .. })
    }

    pub fn is_inconsistent(&self) -> bool {
        matches!(self, AccountChainState::Inconsistent { .. })
    }

    pub fn delegation_record(&self) -> Option<&DelegationRecord> {
        match self {
            AccountChainState::Delegated {
                delegation_record, ..
            } => Some(delegation_record),
            _ => None,
        }
    }

    pub fn account(&self) -> Option<&Account> {
        match self {
            AccountChainState::NewAccount => None,
            AccountChainState::Undelegated { account } => Some(account),
            AccountChainState::Delegated { account, .. } => Some(account),
            AccountChainState::Inconsistent { account, .. } => Some(account),
        }
    }
}

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::transaction_accounts_snapshot::TransactionAccountsSnapshot;

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnroutableReason {
    ContainsBothUndelegatedAndDelegatedAccountsAsWritable {
        writable_undelegated_pubkeys: Vec<Pubkey>,
        writable_delegated_pubkeys: Vec<Pubkey>,
    },
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Endpoint {
    Chain {
        transaction_accounts_snapshot: TransactionAccountsSnapshot,
    },
    Ephemeral {
        transaction_accounts_snapshot: TransactionAccountsSnapshot,
    },
    Unroutable {
        transaction_accounts_snapshot: TransactionAccountsSnapshot,
        reason: UnroutableReason,
    },
}

impl Endpoint {
    pub fn is_ephemeral(&self) -> bool {
        matches!(self, Endpoint::Ephemeral { .. })
    }
    pub fn is_chain(&self) -> bool {
        matches!(self, Endpoint::Chain { .. })
    }
    pub fn is_unroutable(&self) -> bool {
        matches!(self, Endpoint::Unroutable { .. })
    }

    pub fn transaction_accounts_snapshot(
        &self,
    ) -> &TransactionAccountsSnapshot {
        match self {
            Endpoint::Chain {
                transaction_accounts_snapshot,
                ..
            } => transaction_accounts_snapshot,
            Endpoint::Ephemeral {
                transaction_accounts_snapshot,
                ..
            } => transaction_accounts_snapshot,
            Endpoint::Unroutable {
                transaction_accounts_snapshot,
                ..
            } => transaction_accounts_snapshot,
        }
    }
}

impl Endpoint {
    pub fn from(
        transaction_accounts_snapshot: TransactionAccountsSnapshot,
    ) -> Endpoint {
        let writable_undelegated_pubkeys =
            transaction_accounts_snapshot.writable_undelegated_pubkeys();
        let writable_delegated_pubkeys =
            transaction_accounts_snapshot.writable_delegated_pubkeys();

        let has_writable_undelegated = !writable_undelegated_pubkeys.is_empty();
        let has_writable_delegated = !writable_delegated_pubkeys.is_empty();

        match (has_writable_undelegated, has_writable_delegated) {
            // If there are both data and delegated accounts as writable, its not possible to route
            (true, true) => Endpoint::Unroutable {
                transaction_accounts_snapshot,
                reason: UnroutableReason::ContainsBothUndelegatedAndDelegatedAccountsAsWritable {
                    writable_undelegated_pubkeys,
                    writable_delegated_pubkeys,
                },
            },
            // If there is neither delegated nor data accounts as writable, just default to chain
            (false, false) => Endpoint::Chain {
                transaction_accounts_snapshot,
            },
            // If there are only data accounts as writable, its for the chain
            (true, false) => Endpoint::Chain {
                transaction_accounts_snapshot,
            },
            // If there are only delegated accounts as writable, its for the ephemeral
            (false, true) => Endpoint::Ephemeral {
                transaction_accounts_snapshot,
            }
        }
    }
}

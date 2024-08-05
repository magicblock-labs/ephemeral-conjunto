use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::transaction_accounts_snapshot::TransactionAccountsSnapshot;

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnroutableReason {
    WritablesIncludeInconsistentAccounts {
        writable_inconsistent_pubkeys: Vec<Pubkey>,
    },
    ContainsWritableDelegatedAndWritableUndelegated {
        writable_delegated_pubkeys: Vec<Pubkey>,
        writable_undelegated_non_payer_pubkeys: Vec<Pubkey>,
    },
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Endpoint {
    Chain {
        transaction_accounts_snapshot: TransactionAccountsSnapshot,
    },
    Ephemeral {
        transaction_accounts_snapshot: TransactionAccountsSnapshot,
        writable_delegated_pubkeys: Vec<Pubkey>,
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
        // If any account is in an inconsistent delegation state, we can't do anything
        let writable_inconsistent_pubkeys =
            transaction_accounts_snapshot.writable_inconsistent_pubkeys();
        let has_writable_inconsistent =
            !writable_inconsistent_pubkeys.is_empty();
        if has_writable_inconsistent {
            return Endpoint::Unroutable {
                transaction_accounts_snapshot,
                reason:
                    UnroutableReason::WritablesIncludeInconsistentAccounts {
                        writable_inconsistent_pubkeys,
                    },
            };
        }

        // If there are no writable delegated accounts in the transaction, we can route to chain
        let writable_delegated_pubkeys =
            transaction_accounts_snapshot.writable_delegated_pubkeys();
        let has_writable_delegated = !writable_delegated_pubkeys.is_empty();
        if !has_writable_delegated {
            return Endpoint::Chain {
                transaction_accounts_snapshot,
            };
        }

        // At this point, we are planning to route to ephemeral,
        // so there cannot be any writable undelegated except the payer
        // If there are, we cannot route this transaction
        let writable_undelegated_non_payer_pubkeys =
            transaction_accounts_snapshot
                .writable_undelegated_non_payer_pubkeys();
        let has_writable_undelegated_non_payer =
            !writable_undelegated_non_payer_pubkeys.is_empty();
        if has_writable_undelegated_non_payer {
            return Endpoint::Unroutable {
                transaction_accounts_snapshot,
                reason: UnroutableReason::ContainsWritableDelegatedAndWritableUndelegated {
                    writable_delegated_pubkeys,
                    writable_undelegated_non_payer_pubkeys,
                },
            };
        }

        // Now we know that there are only delegated writables
        // or payers that are writable
        Endpoint::Ephemeral {
            transaction_accounts_snapshot,
            writable_delegated_pubkeys,
        }
    }
}

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::transaction_account_meta::TransactionAccountMetas;

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
    Chain(TransactionAccountMetas),
    Ephemeral(TransactionAccountMetas),
    Unroutable {
        account_metas: TransactionAccountMetas,
        reason: UnroutableReason,
    },
}

impl Endpoint {
    pub fn is_ephemeral(&self) -> bool {
        matches!(self, Endpoint::Ephemeral(_))
    }
    pub fn is_chain(&self) -> bool {
        matches!(self, Endpoint::Chain(_))
    }
    pub fn is_unroutable(&self) -> bool {
        matches!(self, Endpoint::Unroutable { .. })
    }
    pub fn into_account_metas(self) -> TransactionAccountMetas {
        use Endpoint::*;
        match self {
            Chain(account_metas)
            | Ephemeral(account_metas)
            | Unroutable { account_metas, .. } => account_metas,
        }
    }
}

impl Endpoint {
    pub fn from(metas: TransactionAccountMetas) -> Endpoint {
        // If any account is in an inconsistent delegation state, we can't do anything
        let writable_inconsistent_pubkeys =
            metas.writable_inconsistent_pubkeys();
        if !writable_inconsistent_pubkeys.is_empty() {
            return Endpoint::Unroutable {
                account_metas: metas,
                reason:
                    UnroutableReason::WritablesIncludeInconsistentAccounts {
                        writable_inconsistent_pubkeys,
                    },
            };
        }

        // If there are no writable delegated accounts in the transaction, we can route to chain
        let writable_delegated_pubkeys = metas.writable_delegated_pubkeys();
        if writable_delegated_pubkeys.is_empty() {
            return Endpoint::Chain(metas);
        }

        let writable_undelegated_non_payer_pubkeys =
            metas.writable_undelegated_non_payer_pubkeys();

        // At this point, we are planning to route to ephemeral,
        // so there cannot be any writable undelegated except the payer
        // If there are, we cannot route this transaction
        let has_writable_undelegated_non_payer =
            !writable_undelegated_non_payer_pubkeys.is_empty();
        if has_writable_undelegated_non_payer {
            return Endpoint::Unroutable {
                account_metas: metas,
                reason: UnroutableReason::ContainsWritableDelegatedAndWritableUndelegated {
                    writable_delegated_pubkeys,
                    writable_undelegated_non_payer_pubkeys,
                },
            };
        }

        // Now we know that there are only delegated writables
        // or payers that are writable
        Endpoint::Ephemeral(metas)
    }
}

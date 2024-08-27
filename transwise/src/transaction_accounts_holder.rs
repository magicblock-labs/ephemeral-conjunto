use solana_sdk::{
    pubkey::Pubkey,
    transaction::{SanitizedTransaction, VersionedTransaction},
};

use crate::errors::{TranswiseError, TranswiseResult};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionAccountsHolder {
    pub writable: Vec<Pubkey>,
    pub readonly: Vec<Pubkey>,
    pub payer: Pubkey,
}

impl TryFrom<&SanitizedTransaction> for TransactionAccountsHolder {
    type Error = TranswiseError;

    fn try_from(tx: &SanitizedTransaction) -> TranswiseResult<Self> {
        let loaded = tx.get_account_locks_unchecked();
        let writable = loaded.writable.iter().map(|x| **x).collect();
        let readonly = loaded.readonly.iter().map(|x| **x).collect();
        let payer = tx
            .message()
            .account_keys()
            .get(0)
            .ok_or(TranswiseError::TransactionIsMissingPayerAccount)?;
        Ok(Self {
            writable,
            readonly,
            payer: *payer,
        })
    }
}

impl TryFrom<&VersionedTransaction> for TransactionAccountsHolder {
    type Error = TranswiseError;
    fn try_from(tx: &VersionedTransaction) -> TranswiseResult<Self> {
        let static_accounts = tx.message.static_account_keys();
        let mut writable = Vec::new();
        let mut readonly = Vec::new();
        let payer = static_accounts
            .first()
            .ok_or(TranswiseError::TransactionIsMissingPayerAccount)?;

        for (idx, pubkey) in static_accounts.iter().enumerate() {
            if tx.message.is_maybe_writable(idx) {
                writable.push(*pubkey);
            } else {
                readonly.push(*pubkey);
            }
        }

        let lookups = tx.message.address_table_lookups().unwrap_or_default();
        for lookup in lookups {
            let _writable_idxs = &lookup.writable_indexes;
            let _readonly_idxs = &lookup.readonly_indexes;
            // TODO(thlorenz): to properly support lookup tables we'd now have to do the following:
            //
            // 1. Fetch data of the lookup table
            // 2. resolve the indexes to actual account keys
            //
            // However to do that there are two issues with this:
            // 1. This method would have to be async and fetching that data results in more latency
            // 2. Where do we fetch the table from, ephemeral or chain? Or first ephemeral and then chain?
            //    The latter would result in even more latency.
        }

        Ok(Self {
            writable,
            readonly,
            payer: *payer,
        })
    }
}

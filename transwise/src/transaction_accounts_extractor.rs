use solana_sdk::transaction::{SanitizedTransaction, VersionedTransaction};

use crate::{
    errors::TranswiseResult,
    transaction_accounts_holder::TransactionAccountsHolder,
};

pub trait TransactionAccountsExtractor {
    fn try_accounts_from_versioned_transaction(
        &self,
        tx: &VersionedTransaction,
    ) -> TranswiseResult<TransactionAccountsHolder>;

    fn try_accounts_from_sanitized_transaction(
        &self,
        tx: &SanitizedTransaction,
    ) -> TranswiseResult<TransactionAccountsHolder>;
}

pub struct TransactionAccountsExtractorImpl;

impl TransactionAccountsExtractor for TransactionAccountsExtractorImpl {
    fn try_accounts_from_versioned_transaction(
        &self,
        tx: &VersionedTransaction,
    ) -> TranswiseResult<TransactionAccountsHolder> {
        TransactionAccountsHolder::try_from(tx)
    }

    fn try_accounts_from_sanitized_transaction(
        &self,
        tx: &SanitizedTransaction,
    ) -> TranswiseResult<TransactionAccountsHolder> {
        TransactionAccountsHolder::try_from(tx)
    }
}

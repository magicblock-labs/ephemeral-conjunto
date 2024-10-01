use crate::{
    errors::{TranswiseError, TranswiseResult},
    transaction_accounts_snapshot::TransactionAccountsSnapshot,
};

pub trait TransactionAccountsValidator {
    /// Read information on the provided accounts,
    /// validates that we will accept this transaction in an ephemeral validator
    /// The checks make sure that all writable accounts are delegated properly
    fn validate_ephemeral_transaction_accounts(
        &self,
        transaction_accounts: &TransactionAccountsSnapshot,
    ) -> TranswiseResult<()>;
}

pub struct TransactionAccountsValidatorImpl;

impl TransactionAccountsValidator for TransactionAccountsValidatorImpl {
    fn validate_ephemeral_transaction_accounts(
        &self,
        transaction_accounts: &TransactionAccountsSnapshot,
    ) -> TranswiseResult<()> {
        // We need make sure that none of the writables are data accounts
        let writable_undelegated_pubkeys =
            transaction_accounts.writable_undelegated_pubkeys();
        let has_writable_undelegated = !writable_undelegated_pubkeys.is_empty();
        if has_writable_undelegated {
            let writable_undelegated_pubkeys =
                transaction_accounts.writable_undelegated_pubkeys();
            return Err(
                TranswiseError::TransactionIncludeUndelegatedAccountsAsWritable {
                    writable_undelegated_pubkeys,
                },
            );
        }
        // Transaction should work fine in other cases
        Ok(())
    }
}

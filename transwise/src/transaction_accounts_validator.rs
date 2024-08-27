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
        // First, a quick guard against writable accounts that are inconsistently delegated
        let writable_inconsistent_pubkeys =
            transaction_accounts.writable_inconsistent_pubkeys();
        let has_writable_inconsistent =
            !writable_inconsistent_pubkeys.is_empty();
        if has_writable_inconsistent {
            return Err(TranswiseError::WritablesIncludeInconsistentAccounts {
                writable_inconsistent_pubkeys,
            });
        }
        // Since new accounts cannot be delegated, we should not accept those in our validator as writable
        let writable_new_pubkeys = transaction_accounts.writable_new_pubkeys();
        let has_writable_new = !writable_new_pubkeys.is_empty();
        if has_writable_new {
            return Err(TranswiseError::WritablesIncludeNewAccounts {
                writable_new_pubkeys,
            });
        }
        // We need make sure that all writables are delegated
        // Except we don't worry about the payer, because it doesn't contain data, it just need to sign
        let writable_undelegated_non_payer_pubkeys =
            transaction_accounts.writable_undelegated_non_payer_pubkeys();
        let has_writable_undelegated_non_payer =
            !writable_undelegated_non_payer_pubkeys.is_empty();
        if has_writable_undelegated_non_payer {
            let writable_delegated_pubkeys =
                transaction_accounts.writable_delegated_pubkeys();
            return Err(TranswiseError::NotAllWritablesDelegated {
                writable_delegated_pubkeys,
                writable_undelegated_non_payer_pubkeys,
            });
        }
        // Transaction should work fine in other cases
        Ok(())
    }
}

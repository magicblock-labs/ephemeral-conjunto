use crate::{
    errors::{TranswiseError, TranswiseResult},
    transaction_accounts_snapshot::TransactionAccountsSnapshot,
};

#[derive(Debug)]
pub struct ValidateAccountsConfig {
    pub allow_new_accounts: bool,
    pub require_delegation: bool,
}

impl Default for ValidateAccountsConfig {
    fn default() -> Self {
        Self {
            allow_new_accounts: false,
            require_delegation: true,
        }
    }
}

pub trait TransactionAccountsValidator {
    /// Read information on the provided accounts, validates that we will accept this transaction
    /// The checks make sure that all writable accounts are either delegated
    /// or conform to what's specified in the config.
    fn validate_accounts(
        &self,
        transaction_accounts: &TransactionAccountsSnapshot,
        config: &ValidateAccountsConfig,
    ) -> TranswiseResult<()>;
}

pub struct TransactionAccountsValidatorImpl;

impl TransactionAccountsValidator for TransactionAccountsValidatorImpl {
    fn validate_accounts(
        &self,
        transaction_accounts: &TransactionAccountsSnapshot,
        config: &ValidateAccountsConfig,
    ) -> TranswiseResult<()> {
        // We put the following constraint on the config:
        //
        // A) the validator CAN create new accounts and can clone ANY account from chain, even non-delegated ones (permissive mode)
        // B) the validator CANNOT create new accounts and can ONLY clone delegated accounts from chain (strict mode)
        // C) the validator CANNOT create new accounts and can clone ANY account from chain, even non-delegated ones (frozen mode)
        //
        // This means we disallow the following remaining case:
        //
        // D) the validator CAN create new accounts and can ONLY clone delegated accounts from chain
        // This edge case is difficult to handle properly and most likely not what the user intended for the following reason:
        // If a transaction has a writable account that does not exist on chain by definition that account is not delegated
        // and if we accept it as a writable it now violates the delegation requirement.
        // In short this is a conflicting requirement that we don't allow.
        if config.require_delegation && config.allow_new_accounts {
            return Err(TranswiseError::ValidateAccountsConfigIsInvalid(
                format!("{:?}", config),
            ));
        }

        // First, a quick guard against accounts that are inconsistently delegated
        let writable_inconsistent_pubkeys =
            transaction_accounts.writable_inconsistent_pubkeys();
        let has_writable_inconsistent =
            !writable_inconsistent_pubkeys.is_empty();
        if has_writable_inconsistent {
            return Err(TranswiseError::WritablesIncludeInconsistentAccounts {
                writable_inconsistent_pubkeys,
            });
        }

        // If we are not allowed to create new accounts, we need to guard against them
        if !config.allow_new_accounts {
            let writable_new_pubkeys =
                transaction_accounts.writable_new_pubkeys();
            let has_writable_new = !writable_new_pubkeys.is_empty();
            if has_writable_new {
                return Err(TranswiseError::WritablesIncludeNewAccounts {
                    writable_new_pubkeys,
                });
            }
        }

        // If we require delegation:
        // We need make sure that all writables are delegated
        // Except we don't worry about the payer, because it doesn't contain data, it just need to sign
        if config.require_delegation {
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
        }

        // Transaction should work fine in other cases
        Ok(())
    }
}

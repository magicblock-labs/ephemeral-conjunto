use async_trait::async_trait;

use crate::{
    errors::TranswiseResult,
    transaction_accounts_holder::TransactionAccountsHolder,
    transwise::Transwise,
    validated_accounts::{ValidateAccountsConfig, ValidatedAccounts},
};

#[async_trait]
pub trait ValidatedAccountsProvider {
    /// Extracts information of the provided accounts, validates
    /// them and returns the result containing writable and readonly accounts.
    /// The checks make sure that all writable accounts are either delegated
    /// or conform to what's specified in the config.
    async fn validate_accounts(
        &self,
        transaction_accounts: &TransactionAccountsHolder,
        config: &ValidateAccountsConfig,
    ) -> TranswiseResult<ValidatedAccounts>;
}

#[async_trait]
impl ValidatedAccountsProvider for Transwise {
    async fn validate_accounts(
        &self,
        transaction_accounts: &TransactionAccountsHolder,
        config: &ValidateAccountsConfig,
    ) -> TranswiseResult<ValidatedAccounts> {
        let account_metas = self.account_metas(transaction_accounts).await?;
        ValidatedAccounts::try_from((account_metas, config))
    }
}

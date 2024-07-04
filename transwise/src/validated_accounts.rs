pub use conjunto_lockbox::LockConfig;
use solana_sdk::{account::Account, pubkey::Pubkey};

use crate::{
    errors::TranswiseError,
    transaction_account_meta::{
        TransactionAccountMeta, TransactionAccountMetas,
    },
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

#[derive(Debug)]
pub struct ValidatedReadonlyAccount {
    pub pubkey: Pubkey,
    pub account: Option<Account>,
}

impl TryFrom<TransactionAccountMeta> for ValidatedReadonlyAccount {
    type Error = TranswiseError;
    fn try_from(
        meta: TransactionAccountMeta,
    ) -> Result<ValidatedReadonlyAccount, Self::Error> {
        match meta {
            TransactionAccountMeta::Readonly {
                pubkey,
                chain_state,
            } => Ok(ValidatedReadonlyAccount {
                pubkey,
                account: chain_state.into_account(),
            }),
            _ => Err(TranswiseError::CreateValidatedReadonlyAccountFailed(
                format!("{:?}", meta),
            )),
        }
    }
}

#[derive(Debug)]
pub struct ValidatedWritableAccount {
    pub pubkey: Pubkey,
    pub account: Option<Account>,
    pub lock_config: Option<LockConfig>,
    pub is_payer: bool,
}

impl TryFrom<TransactionAccountMeta> for ValidatedWritableAccount {
    type Error = TranswiseError;
    fn try_from(
        meta: TransactionAccountMeta,
    ) -> Result<ValidatedWritableAccount, Self::Error> {
        match meta {
            TransactionAccountMeta::Writable {
                pubkey,
                chain_state,
                is_payer,
            } => Ok(ValidatedWritableAccount {
                pubkey,
                lock_config: chain_state.lock_config(),
                account: chain_state.into_account(),
                is_payer,
            }),
            _ => Err(TranswiseError::CreateValidatedWritableAccountFailed(
                format!("{:?}", meta),
            )),
        }
    }
}

#[derive(Debug)]
pub struct ValidatedAccounts {
    pub readonly: Vec<ValidatedReadonlyAccount>,
    pub writable: Vec<ValidatedWritableAccount>,
}

impl TryFrom<(TransactionAccountMetas, &ValidateAccountsConfig)>
    for ValidatedAccounts
{
    type Error = TranswiseError;

    fn try_from(
        (metas, config): (TransactionAccountMetas, &ValidateAccountsConfig),
    ) -> Result<Self, Self::Error> {
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
            metas.writable_inconsistent_pubkeys();
        let has_writable_inconsistent =
            !writable_inconsistent_pubkeys.is_empty();
        if has_writable_inconsistent {
            return Err(TranswiseError::WritablesIncludeInconsistentAccounts {
                writable_inconsistent_pubkeys,
            });
        }

        // If we are not allowed to create new accounts, we need to guard against them
        if !config.allow_new_accounts {
            let writable_new_pubkeys = metas.writable_new_pubkeys();
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
                metas.writable_undelegated_non_payer_pubkeys();
            let has_writable_undelegated_non_payer =
                !writable_undelegated_non_payer_pubkeys.is_empty();
            if has_writable_undelegated_non_payer {
                let writable_delegated_pubkeys =
                    metas.writable_delegated_pubkeys();
                return Err(TranswiseError::NotAllWritablesDelegated {
                    writable_delegated_pubkeys,
                    writable_undelegated_non_payer_pubkeys,
                });
            }
        }

        // NOTE: when we don't require delegation then we still query the account states to
        // get the chain_state of each delegated. This causes some unnecessary overhead which we
        // could avoid if we make the lockbox aware of this, i.e. by adding an LockstateUnknown
        // variant and returning that instead of checking it.
        // However this is only the case when developing locally and thus we may not optimize for it.

        // Generate the validated account structs
        let (readonly_metas, writable_metas): (Vec<_>, Vec<_>) =
            metas.0.into_iter().partition(|meta| match meta {
                TransactionAccountMeta::Readonly { .. } => true,
                TransactionAccountMeta::Writable { .. } => false,
            });

        let validated_readonly_accounts = readonly_metas
            .into_iter()
            .map(ValidatedReadonlyAccount::try_from)
            .collect::<Result<Vec<_>, TranswiseError>>()?;
        let validated_writable_accounts = writable_metas
            .into_iter()
            .map(ValidatedWritableAccount::try_from)
            .collect::<Result<Vec<_>, TranswiseError>>()?;

        // Done
        Ok(ValidatedAccounts {
            readonly: validated_readonly_accounts,
            writable: validated_writable_accounts,
        })
    }
}

#[cfg(test)]
mod tests {
    use conjunto_core::CommitFrequency;
    use conjunto_lockbox::{AccountChainState, LockConfig};
    use conjunto_test_tools::accounts::{
        account_owned_by_delegation_program, account_owned_by_system_program,
    };

    use super::*;
    use crate::{
        errors::TranswiseResult,
        transaction_account_meta::TransactionAccountMeta,
    };

    fn config_strict() -> ValidateAccountsConfig {
        ValidateAccountsConfig {
            allow_new_accounts: false,
            require_delegation: true,
        }
    }

    fn config_permissive() -> ValidateAccountsConfig {
        ValidateAccountsConfig {
            allow_new_accounts: true,
            require_delegation: false,
        }
    }

    fn chain_state_delegated() -> AccountChainState {
        AccountChainState::Delegated {
            account: account_owned_by_delegation_program(),
            delegated_id: Pubkey::new_unique(),
            delegation_pda: Pubkey::new_unique(),
            config: LockConfig {
                commit_frequency: CommitFrequency::Millis(1_000),
                owner: Pubkey::new_unique(),
            },
        }
    }

    fn chain_state_undelegated() -> AccountChainState {
        AccountChainState::Undelegated {
            account: account_owned_by_system_program(),
        }
    }

    fn chain_state_new_account() -> AccountChainState {
        AccountChainState::NewAccount
    }

    fn chain_state_inconsistent() -> AccountChainState {
        AccountChainState::Inconsistent {
            account: account_owned_by_system_program(),
            delegated_id: Pubkey::new_unique(),
            delegation_pda: Pubkey::new_unique(),
            inconsistencies: vec![],
        }
    }

    fn readonly_pubkeys(vas: &ValidatedAccounts) -> Vec<Pubkey> {
        vas.readonly.iter().map(|x| x.pubkey).collect()
    }

    fn writable_pubkeys(vas: &ValidatedAccounts) -> Vec<Pubkey> {
        vas.writable.iter().map(|x| x.pubkey).collect()
    }

    #[test]
    fn test_two_readonly_undelegated_and_two_writable_delegated_and_payer() {
        let readonly_undelegated_id1 = Pubkey::new_unique();
        let readonly_undelegated_id2 = Pubkey::new_unique();
        let writable_delegated_id1 = Pubkey::new_unique();
        let writable_delegated_id2 = Pubkey::new_unique();
        let writable_undelegated_payer_id = Pubkey::new_unique();

        let meta1 = TransactionAccountMeta::Readonly {
            pubkey: readonly_undelegated_id1,
            chain_state: chain_state_undelegated(),
        };
        let meta2 = TransactionAccountMeta::Readonly {
            pubkey: readonly_undelegated_id2,
            chain_state: chain_state_undelegated(),
        };
        let meta3 = TransactionAccountMeta::Writable {
            pubkey: writable_delegated_id1,
            chain_state: chain_state_delegated(),
            is_payer: false,
        };
        let meta4 = TransactionAccountMeta::Writable {
            pubkey: writable_delegated_id2,
            chain_state: chain_state_delegated(),
            is_payer: false,
        };
        let meta5 = TransactionAccountMeta::Writable {
            pubkey: writable_undelegated_payer_id,
            chain_state: chain_state_undelegated(),
            is_payer: true,
        };

        let vas: ValidatedAccounts = (
            TransactionAccountMetas(vec![meta1, meta2, meta3, meta4, meta5]),
            &config_strict(),
        )
            .try_into()
            .unwrap();

        assert_eq!(
            readonly_pubkeys(&vas),
            vec![readonly_undelegated_id1, readonly_undelegated_id2]
        );
        assert_eq!(
            writable_pubkeys(&vas),
            vec![
                writable_delegated_id1,
                writable_delegated_id2,
                writable_undelegated_payer_id
            ]
        );
    }

    #[test]
    fn test_one_readonly_undelegated_and_one_writable_undelegated_fail() {
        let readonly_undelegated_id = Pubkey::new_unique();
        let writable_undelegated_id = Pubkey::new_unique();

        let meta1 = TransactionAccountMeta::Readonly {
            pubkey: readonly_undelegated_id,
            chain_state: chain_state_undelegated(),
        };
        let meta2 = TransactionAccountMeta::Writable {
            pubkey: writable_undelegated_id,
            chain_state: chain_state_undelegated(),
            is_payer: false,
        };

        let res: TranswiseResult<ValidatedAccounts> = (
            TransactionAccountMetas(vec![meta1, meta2]),
            &config_strict(),
        )
            .try_into();

        assert!(res.is_err());
    }

    #[test]
    fn test_one_readonly_undelegated_and_payer() {
        let readonly_undelegated_id = Pubkey::new_unique();
        let writable_undelegated_payer_id = Pubkey::new_unique();

        let meta1 = TransactionAccountMeta::Readonly {
            pubkey: readonly_undelegated_id,
            chain_state: chain_state_undelegated(),
        };
        let meta2 = TransactionAccountMeta::Writable {
            pubkey: writable_undelegated_payer_id,
            chain_state: chain_state_undelegated(),
            is_payer: true,
        };

        let vas: ValidatedAccounts = (
            TransactionAccountMetas(vec![meta1, meta2]),
            &config_strict(),
        )
            .try_into()
            .unwrap();

        assert_eq!(readonly_pubkeys(&vas), vec![readonly_undelegated_id]);
        assert_eq!(writable_pubkeys(&vas), vec![writable_undelegated_payer_id]);
    }

    #[test]
    fn test_one_readonly_undelegated_and_one_writable_inconsistent() {
        let readonly_undelegated_id = Pubkey::new_unique();
        let writable_inconsistent_id = Pubkey::new_unique();

        let meta1 = TransactionAccountMeta::Readonly {
            pubkey: readonly_undelegated_id,
            chain_state: chain_state_undelegated(),
        };
        let meta2 = TransactionAccountMeta::Writable {
            pubkey: writable_inconsistent_id,
            chain_state: chain_state_inconsistent(),
            is_payer: false,
        };

        let res: TranswiseResult<ValidatedAccounts> = (
            TransactionAccountMetas(vec![meta1, meta2]),
            &config_strict(),
        )
            .try_into();

        assert!(res.is_err());
    }

    #[test]
    fn test_one_readonly_new_account_and_one_payer() {
        let readonly_new_account_id = Pubkey::new_unique();
        let writable_undelegated_payer_id = Pubkey::new_unique();

        let meta1 = TransactionAccountMeta::Readonly {
            pubkey: readonly_new_account_id,
            chain_state: chain_state_new_account(),
        };
        let meta2 = TransactionAccountMeta::Writable {
            pubkey: writable_undelegated_payer_id,
            chain_state: chain_state_delegated(),
            is_payer: true,
        };

        let vas: ValidatedAccounts = (
            TransactionAccountMetas(vec![meta1, meta2]),
            &config_strict(),
        )
            .try_into()
            .unwrap();

        // While this is a new account, it's a readonly so we don't need to write to it, so it's valid
        // However it cannot be cloned, but that last bit of clone filtering will be done in the validator
        assert_eq!(readonly_pubkeys(&vas), vec![readonly_new_account_id]);
        assert_eq!(writable_pubkeys(&vas), vec![writable_undelegated_payer_id]);
    }

    #[test]
    fn test_one_readonly_undelegated_and_one_writable_new_account() {
        let readonly_undelegated_id = Pubkey::new_unique();
        let writable_new_account_id = Pubkey::new_unique();

        let meta1 = TransactionAccountMeta::Readonly {
            pubkey: readonly_undelegated_id,
            chain_state: chain_state_undelegated(),
        };
        let meta2 = TransactionAccountMeta::Writable {
            pubkey: writable_new_account_id,
            chain_state: chain_state_new_account(),
            is_payer: false,
        };

        let res: TranswiseResult<ValidatedAccounts> = (
            TransactionAccountMetas(vec![meta1, meta2]),
            &config_strict(),
        )
            .try_into();

        assert!(res.is_err());
    }

    #[test]
    fn test_one_readonly_undelegated_and_one_writable_new_account_and_one_writable_undelegated_while_permissive(
    ) {
        let readonly_undelegated_id1 = Pubkey::new_unique();
        let writable_new_account_id = Pubkey::new_unique();
        let writable_undelegated_id = Pubkey::new_unique();

        let meta1 = TransactionAccountMeta::Readonly {
            pubkey: readonly_undelegated_id1,
            chain_state: chain_state_undelegated(),
        };
        let meta2 = TransactionAccountMeta::Writable {
            pubkey: writable_new_account_id,
            chain_state: chain_state_new_account(),
            is_payer: false,
        };
        let meta3 = TransactionAccountMeta::Writable {
            pubkey: writable_undelegated_id,
            chain_state: chain_state_delegated(),
            is_payer: false,
        };

        let vas: ValidatedAccounts = (
            TransactionAccountMetas(vec![meta1, meta2, meta3]),
            &config_permissive(),
        )
            .try_into()
            .unwrap();

        assert_eq!(readonly_pubkeys(&vas), vec![readonly_undelegated_id1]);
        assert_eq!(
            writable_pubkeys(&vas),
            vec![writable_new_account_id, writable_undelegated_id]
        );
    }

    #[test]
    fn test_one_of_each_valid_type() {
        let readonly_new_account_id = Pubkey::new_unique();
        let readonly_undelegated_id = Pubkey::new_unique();
        let readonly_delegated_id = Pubkey::new_unique();
        let readonly_inconsistent_id = Pubkey::new_unique();
        let writable_delegated_id = Pubkey::new_unique();

        let meta1 = TransactionAccountMeta::Readonly {
            pubkey: readonly_new_account_id,
            chain_state: chain_state_new_account(),
        };
        let meta2 = TransactionAccountMeta::Readonly {
            pubkey: readonly_undelegated_id,
            chain_state: chain_state_undelegated(),
        };
        let meta3 = TransactionAccountMeta::Readonly {
            pubkey: readonly_delegated_id,
            chain_state: chain_state_delegated(),
        };
        let meta4 = TransactionAccountMeta::Readonly {
            pubkey: readonly_inconsistent_id,
            chain_state: chain_state_inconsistent(),
        };
        let meta5 = TransactionAccountMeta::Writable {
            pubkey: writable_delegated_id,
            chain_state: chain_state_delegated(),
            is_payer: false,
        };

        let vas: ValidatedAccounts = (
            TransactionAccountMetas(vec![meta1, meta2, meta3, meta4, meta5]),
            &config_strict(),
        )
            .try_into()
            .unwrap();

        assert_eq!(vas.readonly.len(), 4);
        assert_eq!(vas.writable.len(), 1);

        assert_eq!(vas.readonly[0].pubkey, readonly_new_account_id);
        assert_eq!(vas.readonly[1].pubkey, readonly_undelegated_id);
        assert_eq!(vas.readonly[2].pubkey, readonly_delegated_id);
        assert_eq!(vas.readonly[3].pubkey, readonly_inconsistent_id);
        assert_eq!(vas.writable[0].pubkey, writable_delegated_id);

        assert!(vas.readonly[0].account.is_none());
        assert!(vas.readonly[1].account.is_some());
        assert!(vas.readonly[2].account.is_some());
        assert!(vas.readonly[3].account.is_some());
        assert!(vas.writable[0].account.is_some());
    }

    #[test]
    fn test_one_of_each_valid_type_while_permissive() {
        let readonly_new_account_id = Pubkey::new_unique();
        let readonly_undelegated_id = Pubkey::new_unique();
        let readonly_delegated_id = Pubkey::new_unique();
        let readonly_inconsistent_id = Pubkey::new_unique();

        let writable_new_account_id = Pubkey::new_unique();
        let writable_undelegated_id = Pubkey::new_unique();
        let writable_delegated_id = Pubkey::new_unique();

        let meta1 = TransactionAccountMeta::Readonly {
            pubkey: readonly_new_account_id,
            chain_state: chain_state_new_account(),
        };
        let meta2 = TransactionAccountMeta::Readonly {
            pubkey: readonly_undelegated_id,
            chain_state: chain_state_undelegated(),
        };
        let meta3 = TransactionAccountMeta::Readonly {
            pubkey: readonly_delegated_id,
            chain_state: chain_state_delegated(),
        };
        let meta4 = TransactionAccountMeta::Readonly {
            pubkey: readonly_inconsistent_id,
            chain_state: chain_state_inconsistent(),
        };

        let meta5 = TransactionAccountMeta::Writable {
            pubkey: writable_new_account_id,
            chain_state: chain_state_new_account(),
            is_payer: false,
        };
        let meta6 = TransactionAccountMeta::Writable {
            pubkey: writable_undelegated_id,
            chain_state: chain_state_undelegated(),
            is_payer: false,
        };
        let meta7 = TransactionAccountMeta::Writable {
            pubkey: writable_delegated_id,
            chain_state: chain_state_delegated(),
            is_payer: false,
        };

        let vas: ValidatedAccounts = (
            TransactionAccountMetas(vec![
                meta1, meta2, meta3, meta4, meta5, meta6, meta7,
            ]),
            &config_permissive(),
        )
            .try_into()
            .unwrap();

        assert_eq!(vas.readonly.len(), 4);
        assert_eq!(vas.writable.len(), 3);

        assert_eq!(vas.readonly[0].pubkey, readonly_new_account_id);
        assert_eq!(vas.readonly[1].pubkey, readonly_undelegated_id);
        assert_eq!(vas.readonly[2].pubkey, readonly_delegated_id);
        assert_eq!(vas.readonly[3].pubkey, readonly_inconsistent_id);

        assert_eq!(vas.writable[0].pubkey, writable_new_account_id);
        assert_eq!(vas.writable[1].pubkey, writable_undelegated_id);
        assert_eq!(vas.writable[2].pubkey, writable_delegated_id);

        assert!(vas.readonly[0].account.is_none());
        assert!(vas.readonly[1].account.is_some());
        assert!(vas.readonly[2].account.is_some());
        assert!(vas.readonly[3].account.is_some());

        assert!(vas.writable[0].account.is_none());
        assert!(vas.writable[1].account.is_some());
        assert!(vas.writable[2].account.is_some());
    }
}

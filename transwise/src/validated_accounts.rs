pub use conjunto_lockbox::LockConfig;
use solana_sdk::pubkey::Pubkey;

use crate::{errors::TranswiseError, trans_account_meta::TransAccountMetas};

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
    pub is_program: Option<bool>,
}

#[derive(Debug)]
pub struct ValidatedWritableAccount {
    pub pubkey: Pubkey,

    /// The config for locked accounts.
    /// This is `None` for unlocked writable accounts.
    pub lock_config: Option<LockConfig>,

    /// Indicates if this account was a payer in the transaction from which
    /// it was extracted.
    pub is_payer: bool,
}

#[derive(Debug)]
pub struct ValidatedAccounts {
    pub readonly: Vec<ValidatedReadonlyAccount>,
    pub writable: Vec<ValidatedWritableAccount>,
}

impl ValidatedAccounts {
    pub fn readonly_pubkeys(&self) -> Vec<Pubkey> {
        self.readonly.iter().map(|x| x.pubkey).collect()
    }
    pub fn writable_pubkeys(&self) -> Vec<Pubkey> {
        self.writable.iter().map(|x| x.pubkey).collect()
    }
}

impl TryFrom<(&TransAccountMetas, &ValidateAccountsConfig)>
    for ValidatedAccounts
{
    type Error = TranswiseError;

    fn try_from(
        (meta, config): (&TransAccountMetas, &ValidateAccountsConfig),
    ) -> Result<Self, Self::Error> {
        let unlocked = meta.unlocked_writables();
        // NOTE: when we don't require delegation then we still query the account states to
        // get the lockstate of each writable. This causes some unnecessary overhead which we
        // could avoid if we make the lockbox aware of this, i.e. by adding an LockstateUnknown
        // variant and returning that instead of checking it.
        // However this is only the case when developing locally and thus we may not optimize for
        // it.
        // We also make an exception for payers of a transaction as those we don't require to be
        // locked, but instead create and fund them.
        let has_non_payer_unlocked = unlocked.iter().any(|x| !x.is_payer);
        if config.require_delegation && has_non_payer_unlocked {
            return Err(TranswiseError::NotAllWritablesLocked {
                locked: meta
                    .locked_writables()
                    .into_iter()
                    .map(|x| x.pubkey)
                    .collect(),
                unlocked: meta
                    .unlocked_writables()
                    .into_iter()
                    .map(|x| x.pubkey)
                    .collect(),
            });
        }

        let inconsistent = meta.inconsistent_writables();
        if !inconsistent.is_empty() {
            return Err(TranswiseError::WritablesIncludeInconsistentAccounts {
                inconsistent: meta
                    .inconsistent_writables()
                    .into_iter()
                    .map(|x| *x.pubkey())
                    .collect(),
            });
        }

        if !config.allow_new_accounts && !meta.new_writables().is_empty() {
            return Err(TranswiseError::WritablesIncludeNewAccounts {
                new_accounts: meta
                    .new_writables()
                    .into_iter()
                    .map(|x| x.pubkey)
                    .collect(),
            });
        }
        Ok(ValidatedAccounts {
            readonly: meta.readonly_accounts(),
            writable: meta.writable_accounts(!config.require_delegation),
        })
    }
}

#[cfg(test)]
mod tests {
    use conjunto_core::CommitFrequency;
    use conjunto_lockbox::{AccountLockState, LockConfig};

    use super::*;
    use crate::{
        errors::TranswiseResult, trans_account_meta::TransAccountMeta,
    };

    fn config_no_new_accounts() -> ValidateAccountsConfig {
        ValidateAccountsConfig {
            allow_new_accounts: false,
            require_delegation: true,
        }
    }

    fn config_allow_new_accounts() -> ValidateAccountsConfig {
        ValidateAccountsConfig {
            allow_new_accounts: true,
            require_delegation: true,
        }
    }

    fn locked() -> AccountLockState {
        AccountLockState::Locked {
            delegated_id: Pubkey::new_unique(),
            delegation_pda: Pubkey::new_unique(),
            config: LockConfig {
                commit_frequency: CommitFrequency::Millis(1_000),
                owner: Pubkey::new_unique(),
            },
        }
    }

    fn unlocked() -> AccountLockState {
        AccountLockState::Unlocked { is_program: false }
    }

    fn new_account() -> AccountLockState {
        AccountLockState::NewAccount
    }

    fn inconsistent() -> AccountLockState {
        AccountLockState::Inconsistent {
            delegated_id: Pubkey::new_unique(),
            delegation_pda: Pubkey::new_unique(),
            inconsistencies: vec![],
        }
    }

    #[test]
    fn test_locked_writable_two_readonly() {
        let readonly_id1 = Pubkey::new_unique();
        let readonly_id2 = Pubkey::new_unique();
        let writable_id = Pubkey::new_unique();

        let meta1 = TransAccountMeta::Readonly {
            pubkey: readonly_id1,
            is_program: None,
        };
        let meta2 = TransAccountMeta::Readonly {
            pubkey: readonly_id2,
            is_program: None,
        };
        let meta3 = TransAccountMeta::Writable {
            pubkey: writable_id,
            lockstate: locked(),
            is_payer: false,
        };

        let vas: ValidatedAccounts = (
            &TransAccountMetas(vec![meta1, meta2, meta3]),
            &config_no_new_accounts(),
        )
            .try_into()
            .unwrap();

        assert_eq!(vas.readonly_pubkeys(), vec![readonly_id1, readonly_id2]);
        assert_eq!(vas.writable_pubkeys(), vec![writable_id]);
    }

    #[test]
    fn test_unlocked_writable_one_readonly() {
        let readonly_id = Pubkey::new_unique();
        let writable_id = Pubkey::new_unique();

        let meta1 = TransAccountMeta::Readonly {
            pubkey: readonly_id,
            is_program: None,
        };
        let meta2 = TransAccountMeta::Writable {
            pubkey: writable_id,
            lockstate: unlocked(),
            is_payer: false,
        };

        let res: TranswiseResult<ValidatedAccounts> = (
            &TransAccountMetas(vec![meta1, meta2]),
            &config_no_new_accounts(),
        )
            .try_into();

        assert!(res.is_err());
    }

    #[test]
    fn test_unlocked_writable_payer_one_readonly() {
        let readonly_id = Pubkey::new_unique();
        let writable_id = Pubkey::new_unique();

        let meta1 = TransAccountMeta::Readonly {
            pubkey: readonly_id,
            is_program: None,
        };
        let meta2 = TransAccountMeta::Writable {
            pubkey: writable_id,
            lockstate: unlocked(),
            is_payer: true,
        };

        let vas: ValidatedAccounts = (
            &TransAccountMetas(vec![meta1, meta2]),
            &config_no_new_accounts(),
        )
            .try_into()
            .unwrap();

        assert_eq!(vas.readonly_pubkeys(), vec![readonly_id]);
        assert_eq!(vas.writable_pubkeys(), vec![writable_id]);
    }

    #[test]
    fn test_inconsistent_writable_one_readonly() {
        let readonly_id = Pubkey::new_unique();
        let writable_id = Pubkey::new_unique();

        let meta1 = TransAccountMeta::Readonly {
            pubkey: readonly_id,
            is_program: None,
        };
        let meta2 = TransAccountMeta::Writable {
            pubkey: writable_id,
            lockstate: inconsistent(),
            is_payer: false,
        };

        let res: TranswiseResult<ValidatedAccounts> = (
            &TransAccountMetas(vec![meta1, meta2]),
            &config_no_new_accounts(),
        )
            .try_into();

        assert!(res.is_err());
    }

    #[test]
    fn test_locked_writable_one_new_writable_one_readonly_allowing_new() {
        let readonly_id1 = Pubkey::new_unique();
        let new_writable_id = Pubkey::new_unique();
        let locked_writable_id = Pubkey::new_unique();

        let meta1 = TransAccountMeta::Readonly {
            pubkey: readonly_id1,
            is_program: None,
        };
        let meta2 = TransAccountMeta::Writable {
            pubkey: new_writable_id,
            lockstate: new_account(),
            is_payer: false,
        };
        let meta3 = TransAccountMeta::Writable {
            pubkey: locked_writable_id,
            lockstate: locked(),
            is_payer: false,
        };

        let vas: ValidatedAccounts = (
            &TransAccountMetas(vec![meta1, meta2, meta3]),
            &config_allow_new_accounts(),
        )
            .try_into()
            .unwrap();

        assert_eq!(vas.readonly_pubkeys(), vec![readonly_id1]);
        assert_eq!(
            vas.writable_pubkeys(),
            vec![locked_writable_id, new_writable_id]
        );
    }
}

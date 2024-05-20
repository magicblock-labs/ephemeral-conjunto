use solana_sdk::pubkey::Pubkey;

use crate::{errors::TranswiseError, trans_account_meta::TransAccountMetas};

#[derive(Debug, Default)]
pub struct ValidateAccountsConfig {
    pub allow_new_accounts: bool,
}

pub struct ValidatedAccounts {
    pub readonly: Vec<Pubkey>,
    pub writable: Vec<Pubkey>,
}

impl TryFrom<(&TransAccountMetas, &ValidateAccountsConfig)>
    for ValidatedAccounts
{
    type Error = TranswiseError;

    fn try_from(
        (meta, config): (&TransAccountMetas, &ValidateAccountsConfig),
    ) -> Result<Self, Self::Error> {
        let unlocked = meta.unlocked_writables();
        if !unlocked.is_empty() {
            return Err(TranswiseError::NotAllWritablesLocked {
                locked: meta
                    .locked_writables()
                    .into_iter()
                    .map(|x| *x.pubkey())
                    .collect(),
                unlocked: meta
                    .unlocked_writables()
                    .into_iter()
                    .map(|x| *x.pubkey())
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
                    .map(|x| *x.pubkey())
                    .collect(),
            });
        }
        Ok(ValidatedAccounts {
            readonly: meta.readable_pubkeys(),
            writable: meta.writable_pubkeys(),
        })
    }
}

#[cfg(test)]
mod tests {
    use conjunto_lockbox::AccountLockState;

    use crate::{
        errors::TranswiseResult, trans_account_meta::TransAccountMeta,
    };

    use super::*;

    fn config_no_new_accounts() -> ValidateAccountsConfig {
        ValidateAccountsConfig {
            allow_new_accounts: false,
        }
    }

    fn config_allow_new_accounts() -> ValidateAccountsConfig {
        ValidateAccountsConfig {
            allow_new_accounts: true,
        }
    }

    fn locked() -> AccountLockState {
        AccountLockState::Locked {
            delegated_id: Pubkey::new_unique(),
            delegation_pda: Pubkey::new_unique(),
        }
    }

    fn unlocked() -> AccountLockState {
        AccountLockState::Unlocked
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

        let meta1 = TransAccountMeta::readonly(readonly_id1);
        let meta2 = TransAccountMeta::readonly(readonly_id2);
        let meta3 = TransAccountMeta::Writable {
            pubkey: writable_id,
            lockstate: locked(),
        };

        let vas: ValidatedAccounts = (
            &TransAccountMetas(vec![meta1, meta2, meta3]),
            &config_no_new_accounts(),
        )
            .try_into()
            .unwrap();

        assert_eq!(vas.readonly, vec![readonly_id1, readonly_id2]);
        assert_eq!(vas.writable, vec![writable_id]);
    }

    #[test]
    fn test_unlocked_writable_one_readonly() {
        let readonly_id = Pubkey::new_unique();
        let writable_id = Pubkey::new_unique();

        let meta1 = TransAccountMeta::readonly(readonly_id);
        let meta2 = TransAccountMeta::Writable {
            pubkey: writable_id,
            lockstate: unlocked(),
        };

        let res: TranswiseResult<ValidatedAccounts> = (
            &TransAccountMetas(vec![meta1, meta2]),
            &config_no_new_accounts(),
        )
            .try_into();

        assert!(res.is_err());
    }

    #[test]
    fn test_inconsistent_writable_one_readonly() {
        let readonly_id = Pubkey::new_unique();
        let writable_id = Pubkey::new_unique();

        let meta1 = TransAccountMeta::readonly(readonly_id);
        let meta2 = TransAccountMeta::Writable {
            pubkey: writable_id,
            lockstate: inconsistent(),
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

        let meta1 = TransAccountMeta::readonly(readonly_id1);
        let meta2 = TransAccountMeta::Writable {
            pubkey: new_writable_id,
            lockstate: new_account(),
        };
        let meta3 = TransAccountMeta::Writable {
            pubkey: locked_writable_id,
            lockstate: locked(),
        };

        let vas: ValidatedAccounts = (
            &TransAccountMetas(vec![meta1, meta2, meta3]),
            &config_allow_new_accounts(),
        )
            .try_into()
            .unwrap();

        assert_eq!(vas.readonly, vec![readonly_id1]);
        assert_eq!(vas.writable, vec![locked_writable_id, new_writable_id]);
    }
}

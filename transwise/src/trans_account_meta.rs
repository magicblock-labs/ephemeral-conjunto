use std::ops::Deref;

use conjunto_core::{AccountProvider, TransactionAccountsHolder};
use conjunto_lockbox::{AccountLockState, AccountLockStateProvider};
use solana_sdk::{pubkey::Pubkey, transaction::SanitizedTransaction};

use crate::errors::TranswiseResult;

// -----------------
// SanitizedTransactionAccountsHolder
// -----------------
pub struct SanitizedTransactionAccountsHolder {
    writable: Vec<Pubkey>,
    readonly: Vec<Pubkey>,
}

impl From<&SanitizedTransaction> for SanitizedTransactionAccountsHolder {
    fn from(tx: &SanitizedTransaction) -> Self {
        let loaded = tx.get_account_locks_unchecked();
        let writable = loaded.writable.iter().map(|x| **x).collect();
        let readonly = loaded.readonly.iter().map(|x| **x).collect();
        Self { writable, readonly }
    }
}
impl TransactionAccountsHolder for SanitizedTransactionAccountsHolder {
    fn get_writable(&self) -> Vec<Pubkey> {
        self.writable.clone()
    }
    fn get_readonly(&self) -> Vec<Pubkey> {
        self.readonly.clone()
    }
}

// -----------------
// TransAccountMeta
// -----------------
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransAccountMeta {
    Writable {
        pubkey: Pubkey,
        lockstate: AccountLockState,
    },
    Readonly {
        pubkey: Pubkey,
    },
}

impl TransAccountMeta {
    pub fn readonly(pubkey: Pubkey) -> Self {
        TransAccountMeta::Readonly { pubkey }
    }

    pub async fn try_writable<T: AccountProvider>(
        pubkey: Pubkey,
        lockbox: &AccountLockStateProvider<T>,
    ) -> TranswiseResult<Self> {
        let lockstate = lockbox.try_lockstate_of_pubkey(&pubkey).await?;
        Ok(TransAccountMeta::Writable { pubkey, lockstate })
    }

    pub fn pubkey(&self) -> &Pubkey {
        use TransAccountMeta::*;
        match self {
            Writable { pubkey, .. } => pubkey,
            Readonly { pubkey } => pubkey,
        }
    }
}

// -----------------
// Endpoint
// -----------------
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Endpoint {
    Chain(TransAccountMetas),
    Ephemeral(TransAccountMetas),
    Unroutable {
        account_metas: TransAccountMetas,
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
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum UnroutableReason {
    InconsistentLocksEncountered {
        inconsistent_writables: Vec<Pubkey>,
    },
    BothLockedAndUnlocked {
        locked_writables: Vec<Pubkey>,
        unlocked_writables: Vec<Pubkey>,
    },
    NoWritableAccounts,
}

// -----------------
// TransAccountMetas
// -----------------
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TransAccountMetas(pub Vec<TransAccountMeta>);

impl Deref for TransAccountMetas {
    type Target = Vec<TransAccountMeta>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TransAccountMetas {
    pub async fn from_sanitized_transaction<T: AccountProvider>(
        tx: &SanitizedTransaction,
        lockbox: &AccountLockStateProvider<T>,
    ) -> TranswiseResult<Self> {
        let tx_accounts = SanitizedTransactionAccountsHolder::from(tx);
        let account_metas =
            Self::from_accounts_holder(&tx_accounts, lockbox).await?;
        Ok(account_metas)
    }

    pub async fn from_accounts_holder<
        T: AccountProvider,
        U: TransactionAccountsHolder,
    >(
        tx: &U,
        lockbox: &AccountLockStateProvider<T>,
    ) -> TranswiseResult<Self> {
        let mut account_metas = Vec::new();
        let readonly = tx.get_readonly();
        let writable = tx.get_writable();
        for pubkey in readonly.into_iter() {
            account_metas.push(TransAccountMeta::readonly(pubkey));
        }
        for pubkey in writable.into_iter() {
            let account_meta =
                TransAccountMeta::try_writable(pubkey, lockbox).await?;
            account_metas.push(account_meta);
        }

        Ok(Self(account_metas))
    }

    pub fn into_endpoint(self) -> Endpoint {
        use Endpoint::*;
        use UnroutableReason::*;

        // If any of the writables are inconsistent, i.e. not fully locked
        // then we need to abort routing
        let inconsistent_writables = self.inconsistent_writables();
        if !inconsistent_writables.is_empty() {
            let inconsistent_pubkeys = inconsistent_writables
                .iter()
                .map(|x| *x.pubkey())
                .collect::<Vec<Pubkey>>();
            return Unroutable {
                account_metas: self,
                reason: InconsistentLocksEncountered {
                    inconsistent_writables: inconsistent_pubkeys,
                },
            };
        }

        let locked_writeables = self.locked_writables();
        let unlocked_writeables = self.unlocked_writables();

        let has_locked_accounts = !locked_writeables.is_empty();
        let has_unlocked_accounts = !unlocked_writeables.is_empty();

        match (has_locked_accounts, has_unlocked_accounts) {
            // If we write to both locked and unlocked accounts that exist on chain
            // then we cannot route it either to the chain or the ephemeral validator
            (true, true) => {
                let locked_pubkeys = locked_writeables
                    .iter()
                    .map(|x| *x.pubkey())
                    .collect::<Vec<Pubkey>>();
                let unlocked_pubkeys = unlocked_writeables
                    .iter()
                    .map(|x| *x.pubkey())
                    .collect::<Vec<Pubkey>>();
                Unroutable {
                    account_metas: self,
                    reason: BothLockedAndUnlocked {
                        locked_writables: locked_pubkeys,
                        unlocked_writables: unlocked_pubkeys,
                    },
                }
            }
            // If all writables are locked we route to our ephemeral validator
            (true, false) => Ephemeral(self),
            // If all writables are unlocked we route to the chain
            (false, true) => Chain(self),
            // If we write to only new accounts we default to routing to the ephemeral
            // for now.
            // TODO(thlorenz): this edge case could be made configurable by having the user include
            //                 a specific account address as readable which signals what to do here
            //                 i.e. 'Ephemeral111111111111111' forces our validator, otherwise we go
            //                 to chain
            _ => {
                // Assert that we at least got some writable account since otherwise the
                // transaction isn't valid and it makes no sense to rout it anywhere
                if self.new_writables().is_empty() {
                    Unroutable {
                        account_metas: self,
                        reason: NoWritableAccounts,
                    }
                } else {
                    Ephemeral(self)
                }
            }
        }
    }

    fn locked_writables(&self) -> Vec<&TransAccountMeta> {
        self
            .iter()
            .filter(|x| matches!(x, TransAccountMeta::Writable { lockstate, .. } if lockstate.is_locked()))
            .collect()
    }

    fn unlocked_writables(&self) -> Vec<&TransAccountMeta> {
        self
            .iter()
            .filter(|x| matches!(x, TransAccountMeta::Writable { lockstate, .. } if lockstate.is_unlocked()))
            .collect()
    }

    fn new_writables(&self) -> Vec<&TransAccountMeta> {
        self
            .iter()
            .filter(|x| matches!(x, TransAccountMeta::Writable { lockstate, .. } if lockstate.is_new()))
            .collect()
    }

    fn inconsistent_writables(&self) -> Vec<&TransAccountMeta> {
        self
            .iter()
            .filter(|x| matches!(x, TransAccountMeta::Writable { lockstate, .. } if lockstate.is_inconsistent()))
            .collect()
    }
}

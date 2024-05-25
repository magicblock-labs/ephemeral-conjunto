use std::ops::Deref;

use conjunto_core::{AccountProvider, AccountsHolder, DelegationRecordParser};
use conjunto_lockbox::{AccountLockState, AccountLockStateProvider};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    transaction::{SanitizedTransaction, VersionedTransaction},
};

use crate::{
    errors::{TranswiseError, TranswiseResult},
    validated_accounts::{ValidatedReadonlyAccount, ValidatedWritableAccount},
};

// -----------------
// SanitizedTransactionAccountsHolder
// -----------------
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

impl AccountsHolder for TransactionAccountsHolder {
    fn get_writable(&self) -> Vec<Pubkey> {
        self.writable.clone()
    }
    fn get_readonly(&self) -> Vec<Pubkey> {
        self.readonly.clone()
    }
    fn get_payer(&self) -> &Pubkey {
        &self.payer
    }
}

// -----------------
// TransAccountMeta
// -----------------
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum TransAccountMeta {
    Writable {
        pubkey: Pubkey,
        lockstate: AccountLockState,
        is_payer: bool,
    },
    /// Readable account.
    /// If it was found on chain the [is_program] flag tells us if it was executable on chain.
    /// If not found [is_program] is None.
    Readonly {
        pubkey: Pubkey,
        is_program: Option<bool>,
    },
}

impl TransAccountMeta {
    pub async fn try_readonly<T: AccountProvider>(
        pubkey: Pubkey,
        account_provider: &T,
    ) -> TranswiseResult<Self> {
        let acc = account_provider.get_account(&pubkey).await?;
        Ok(TransAccountMeta::Readonly {
            pubkey,
            is_program: acc.map(|x| x.executable),
        })
    }

    pub async fn try_writable<T: AccountProvider, U: DelegationRecordParser>(
        pubkey: Pubkey,
        lockbox: &AccountLockStateProvider<T, U>,
        payer: &Pubkey,
    ) -> TranswiseResult<Self> {
        let lockstate = lockbox.try_lockstate_of_pubkey(&pubkey).await?;
        let is_payer = pubkey == *payer;
        Ok(TransAccountMeta::Writable {
            pubkey,
            lockstate,
            is_payer,
        })
    }

    pub fn pubkey(&self) -> &Pubkey {
        use TransAccountMeta::*;
        match self {
            Writable { pubkey, .. } => pubkey,
            Readonly { pubkey, .. } => pubkey,
        }
    }

    pub fn is_payer(&self) -> bool {
        matches!(self, TransAccountMeta::Writable { is_payer: true, .. })
    }
}

// -----------------
// Endpoint
// -----------------
#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
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
    pub fn into_account_metas(self) -> TransAccountMetas {
        use Endpoint::*;
        match self {
            Chain(account_metas)
            | Ephemeral(account_metas)
            | Unroutable { account_metas, .. } => account_metas,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnroutableReason {
    InconsistentLocksEncountered {
        inconsistent_writables: Vec<Pubkey>,
    },
    BothLockedAndUnlocked {
        locked_writables: Vec<Pubkey>,
        unlocked_writables: Vec<Pubkey>,
        unlocked_payers: Vec<Pubkey>,
    },
    NoWritableAccounts,
}

// -----------------
// TransAccountMetas
// -----------------
#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct TransAccountMetas(pub Vec<TransAccountMeta>);

impl Deref for TransAccountMetas {
    type Target = Vec<TransAccountMeta>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TransAccountMetas {
    pub async fn from_versioned_transaction<
        T: AccountProvider,
        U: DelegationRecordParser,
    >(
        tx: &VersionedTransaction,
        lockbox: &AccountLockStateProvider<T, U>,
    ) -> TranswiseResult<Self> {
        let tx_accounts = TransactionAccountsHolder::try_from(tx)?;
        Self::from_accounts_holder(&tx_accounts, lockbox).await
    }

    pub async fn from_sanitized_transaction<
        T: AccountProvider,
        U: DelegationRecordParser,
    >(
        tx: &SanitizedTransaction,
        lockbox: &AccountLockStateProvider<T, U>,
    ) -> TranswiseResult<Self> {
        let tx_accounts = TransactionAccountsHolder::try_from(tx)?;
        Self::from_accounts_holder(&tx_accounts, lockbox).await
    }

    pub async fn from_accounts_holder<
        T: AccountProvider,
        U: AccountsHolder,
        V: DelegationRecordParser,
    >(
        holder: &U,
        lockbox: &AccountLockStateProvider<T, V>,
    ) -> TranswiseResult<Self> {
        let mut account_metas = Vec::new();
        let readonly = holder.get_readonly();
        let writable = holder.get_writable();
        for pubkey in readonly.into_iter() {
            account_metas.push(
                TransAccountMeta::try_readonly(
                    pubkey,
                    lockbox.account_provider(),
                )
                .await?,
            );
        }
        for pubkey in writable.into_iter() {
            let account_meta = TransAccountMeta::try_writable(
                pubkey,
                lockbox,
                holder.get_payer(),
            )
            .await?;
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
        let (payer_unlocked_accounts, non_payer_unlocked_accounts) =
            unlocked_writeables
                .into_iter()
                .partition::<Vec<_>, _>(|x| x.is_payer);
        let has_non_payer_unlocked_accounts =
            !non_payer_unlocked_accounts.is_empty();

        let has_payer_unlocked_accounts = !payer_unlocked_accounts.is_empty();

        match (has_locked_accounts, has_non_payer_unlocked_accounts) {
            // If we write to both locked and unlocked accounts that exist on chain
            // then we cannot route it either to the chain or the ephemeral validator
            // NOTE: this doens't consider the special case in which we allow cloning
            // non-delegated writable accounts, however that should never be the case
            // when the director determines an endpoint
            (true, true) => {
                let locked_pubkeys = locked_writeables
                    .iter()
                    .map(|x| x.pubkey)
                    .collect::<Vec<Pubkey>>();
                let non_payer_unlocked_pubkeys = non_payer_unlocked_accounts
                    .iter()
                    .map(|x| x.pubkey)
                    .collect::<Vec<Pubkey>>();
                let payer_unlocked_pubkeys = payer_unlocked_accounts
                    .iter()
                    .map(|x| x.pubkey)
                    .collect::<Vec<Pubkey>>();
                Unroutable {
                    account_metas: self,
                    reason: BothLockedAndUnlocked {
                        locked_writables: locked_pubkeys,
                        unlocked_writables: non_payer_unlocked_pubkeys,
                        unlocked_payers: payer_unlocked_pubkeys,
                    },
                }
            }
            // If all writables are locked we route to our ephemeral validator
            (true, false) => Ephemeral(self),
            // If all writables are unlocked we route to the chain
            (false, true) => Chain(self),
            _ if has_payer_unlocked_accounts => {
                // If we write to payer unlocked accounts we default to routing
                // to the chain.
                // See transwise/tests/account_meta.rs
                // test_account_meta_one_unlocked_writable_that_is_payer for more info
                Chain(self)
            }
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

    pub fn writable_accounts(
        &self,
        include_unlocked: bool,
    ) -> Vec<ValidatedWritableAccount> {
        let writables = self
            .locked_writables()
            .into_iter()
            .chain(self.new_writables())
            .collect::<Vec<_>>();
        if include_unlocked {
            // Either we include all unlocked accounts
            writables
                .into_iter()
                .chain(self.unlocked_writables())
                .collect()
        } else {
            // Or only the ones that are also payers since we make a special case for them
            writables
                .into_iter()
                .chain(self.unlocked_writable_payers())
                .collect()
        }
    }

    pub fn readonly_accounts(&self) -> Vec<ValidatedReadonlyAccount> {
        self.iter()
            .flat_map(|x| match x {
                TransAccountMeta::Readonly { pubkey, is_program } => {
                    Some(ValidatedReadonlyAccount {
                        pubkey: *pubkey,
                        is_program: *is_program,
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn readonly_non_program_pubkeys(&self) -> Vec<Pubkey> {
        self.iter()
            .filter(|x| {
                matches!(
                    x,
                    TransAccountMeta::Readonly {
                        is_program: Some(false),
                        ..
                    }
                )
            })
            .map(|x| *x.pubkey())
            .collect()
    }

    pub fn readonly_program_pubkeys(&self) -> Vec<Pubkey> {
        self.iter()
            .filter(|x| {
                matches!(
                    x,
                    TransAccountMeta::Readonly {
                        is_program: Some(true),
                        ..
                    }
                )
            })
            .map(|x| *x.pubkey())
            .collect()
    }

    /// All locked writable accounts.
    pub(crate) fn locked_writables(&self) -> Vec<ValidatedWritableAccount> {
        self.iter()
            .flat_map(|x| match x {
                TransAccountMeta::Writable {
                    lockstate: AccountLockState::Locked { config, .. },
                    ..
                } => Some(ValidatedWritableAccount {
                    pubkey: *x.pubkey(),
                    is_payer: x.is_payer(),
                    owner: Some(config.owner),
                }),
                _ => None,
            })
            .collect()
    }

    /// All unlocked writable accounts.
    pub(crate) fn unlocked_writables(&self) -> Vec<ValidatedWritableAccount> {
        self.iter()
            .flat_map(|x| match x {
                TransAccountMeta::Writable { lockstate, .. }
                    if lockstate.is_unlocked() =>
                {
                    Some(ValidatedWritableAccount {
                        pubkey: *x.pubkey(),
                        is_payer: x.is_payer(),
                        owner: None,
                    })
                }
                _ => None,
            })
            .collect()
    }

    /// Unlocked writable accounts that are also payers of the transaction they
    /// were extracted from.
    pub(crate) fn unlocked_writable_payers(
        &self,
    ) -> Vec<ValidatedWritableAccount> {
        self.iter()
            .flat_map(|x| match x {
                TransAccountMeta::Writable {
                    lockstate,
                    is_payer: true,
                    ..
                } if lockstate.is_unlocked() => {
                    Some(ValidatedWritableAccount {
                        pubkey: *x.pubkey(),
                        is_payer: x.is_payer(),
                        owner: None,
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub(crate) fn new_writables(&self) -> Vec<ValidatedWritableAccount> {
        self.iter()
            .flat_map(|x| match x {
                TransAccountMeta::Writable { lockstate, .. }
                    if lockstate.is_new() =>
                {
                    Some(ValidatedWritableAccount {
                        pubkey: *x.pubkey(),
                        is_payer: x.is_payer(),
                        owner: None,
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub(crate) fn inconsistent_writables(&self) -> Vec<&TransAccountMeta> {
        self
            .iter()
            .filter(|x| matches!(x, TransAccountMeta::Writable { lockstate, .. } if lockstate.is_inconsistent()))
            .collect()
    }
}

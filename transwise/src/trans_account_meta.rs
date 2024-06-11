use std::ops::Deref;

use conjunto_core::{AccountProvider, AccountsHolder, DelegationRecordParser};
use conjunto_lockbox::{AccountLockState, AccountLockStateProvider};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    transaction::{SanitizedTransaction, VersionedTransaction},
};

use crate::{
    errors::TranswiseResult,
    transaction_accounts_holder::TransactionAccountsHolder,
};

// TODO(vbrunet) - this abbreviation is a bit confusing, TransactionAccountMeta would be clearer?
// -----------------
// TransAccountMeta
// -----------------
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum TransAccountMeta {
    Readonly {
        pubkey: Pubkey,
        lockstate: AccountLockState,
    },
    Writable {
        pubkey: Pubkey,
        lockstate: AccountLockState,
        is_payer: bool,
    },
}

impl TransAccountMeta {
    pub async fn try_readonly<T: AccountProvider, U: DelegationRecordParser>(
        pubkey: Pubkey,
        lockbox: &AccountLockStateProvider<T, U>,
    ) -> TranswiseResult<Self> {
        let lockstate = lockbox.try_lockstate_of_pubkey(&pubkey).await?;
        Ok(TransAccountMeta::Readonly { pubkey, lockstate })
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
        match self {
            TransAccountMeta::Readonly { pubkey, .. } => pubkey,
            TransAccountMeta::Writable { pubkey, .. } => pubkey,
        }
    }

    pub fn lockstate(&self) -> &AccountLockState {
        match self {
            TransAccountMeta::Readonly { lockstate, .. } => lockstate,
            TransAccountMeta::Writable { lockstate, .. } => lockstate,
        }
    }

    pub fn is_payer(&self) -> bool {
        matches!(self, TransAccountMeta::Writable { is_payer: true, .. })
    }
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
            account_metas
                .push(TransAccountMeta::try_readonly(pubkey, lockbox).await?);
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

    pub fn writable_inconsistent_pubkeys(&self) -> Vec<Pubkey> {
        self.iter()
            .filter(|x| match x {
                TransAccountMeta::Writable { lockstate, .. } => {
                    lockstate.is_inconsistent()
                }
                _ => false,
            })
            .map(|x| *x.pubkey())
            .collect()
    }

    pub fn writable_delegated_pubkeys(&self) -> Vec<Pubkey> {
        self.iter()
            .filter(|x| match x {
                TransAccountMeta::Writable { lockstate, .. } => {
                    lockstate.is_delegated()
                }
                _ => false,
            })
            .map(|x| *x.pubkey())
            .collect()
    }

    pub fn writable_undelegated_non_payer_pubkeys(&self) -> Vec<Pubkey> {
        self.iter()
            .filter(|x| match x {
                TransAccountMeta::Writable {
                    is_payer: false,
                    lockstate,
                    ..
                } => !lockstate.is_delegated(),
                _ => false,
            })
            .map(|x| *x.pubkey())
            .collect()
    }

    pub fn writable_new_pubkeys(&self) -> Vec<Pubkey> {
        self.iter()
            .filter(|x| match x {
                TransAccountMeta::Writable { lockstate, .. } => {
                    lockstate.is_new()
                }
                _ => false,
            })
            .map(|x| *x.pubkey())
            .collect()
    }
}

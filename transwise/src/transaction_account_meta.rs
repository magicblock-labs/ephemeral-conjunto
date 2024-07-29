use std::ops::Deref;

use conjunto_core::{AccountProvider, AccountsHolder, DelegationRecordParser};
use conjunto_lockbox::{AccountChainSnapshot, AccountChainSnapshotProvider};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    transaction::{SanitizedTransaction, VersionedTransaction},
};

use crate::{
    errors::TranswiseResult,
    transaction_accounts_holder::TransactionAccountsHolder,
};

// -----------------
// TransactionAccountMeta
// -----------------
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum TransactionAccountMeta {
    Readonly {
        pubkey: Pubkey,
        chain_snapshot: AccountChainSnapshot,
    },
    Writable {
        pubkey: Pubkey,
        chain_snapshot: AccountChainSnapshot,
        is_payer: bool,
    },
}

impl TransactionAccountMeta {
    pub async fn try_readonly<T: AccountProvider, U: DelegationRecordParser>(
        pubkey: Pubkey,
        account_chain_snapshot_provider: &AccountChainSnapshotProvider<T, U>,
    ) -> TranswiseResult<Self> {
        let chain_snapshot = account_chain_snapshot_provider
            .try_fetch_chain_snapshot_of_pubkey(&pubkey)
            .await?;
        Ok(TransactionAccountMeta::Readonly {
            pubkey,
            chain_snapshot,
        })
    }

    pub async fn try_writable<T: AccountProvider, U: DelegationRecordParser>(
        pubkey: Pubkey,
        account_chain_snapshot_provider: &AccountChainSnapshotProvider<T, U>,
        payer: &Pubkey,
    ) -> TranswiseResult<Self> {
        let chain_snapshot = account_chain_snapshot_provider
            .try_fetch_chain_snapshot_of_pubkey(&pubkey)
            .await?;
        let is_payer = pubkey == *payer;
        Ok(TransactionAccountMeta::Writable {
            pubkey,
            chain_snapshot,
            is_payer,
        })
    }

    pub fn pubkey(&self) -> &Pubkey {
        match self {
            TransactionAccountMeta::Readonly { pubkey, .. } => pubkey,
            TransactionAccountMeta::Writable { pubkey, .. } => pubkey,
        }
    }

    pub fn chain_snapshot(&self) -> &AccountChainSnapshot {
        match self {
            TransactionAccountMeta::Readonly { chain_snapshot, .. } => {
                chain_snapshot
            }
            TransactionAccountMeta::Writable { chain_snapshot, .. } => {
                chain_snapshot
            }
        }
    }

    pub fn is_payer(&self) -> bool {
        matches!(
            self,
            TransactionAccountMeta::Writable { is_payer: true, .. }
        )
    }
}

// -----------------
// TransactionAccountMetas
// -----------------
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct TransactionAccountMetas(pub Vec<TransactionAccountMeta>);

impl Deref for TransactionAccountMetas {
    type Target = Vec<TransactionAccountMeta>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TransactionAccountMetas {
    pub async fn from_versioned_transaction<
        T: AccountProvider,
        U: DelegationRecordParser,
    >(
        tx: &VersionedTransaction,
        account_chain_snapshot_provider: &AccountChainSnapshotProvider<T, U>,
    ) -> TranswiseResult<Self> {
        let tx_accounts = TransactionAccountsHolder::try_from(tx)?;
        Self::from_accounts_holder(
            &tx_accounts,
            account_chain_snapshot_provider,
        )
        .await
    }

    pub async fn from_sanitized_transaction<
        T: AccountProvider,
        U: DelegationRecordParser,
    >(
        tx: &SanitizedTransaction,
        account_chain_snapshot_provider: &AccountChainSnapshotProvider<T, U>,
    ) -> TranswiseResult<Self> {
        let tx_accounts = TransactionAccountsHolder::try_from(tx)?;
        Self::from_accounts_holder(
            &tx_accounts,
            account_chain_snapshot_provider,
        )
        .await
    }

    pub async fn from_accounts_holder<
        T: AccountProvider,
        U: AccountsHolder,
        V: DelegationRecordParser,
    >(
        holder: &U,
        account_chain_snapshot_provider: &AccountChainSnapshotProvider<T, V>,
    ) -> TranswiseResult<Self> {
        let mut account_metas = Vec::new();
        let readonly = holder.get_readonly();
        let writable = holder.get_writable();
        for pubkey in readonly.into_iter() {
            account_metas.push(
                TransactionAccountMeta::try_readonly(
                    pubkey,
                    account_chain_snapshot_provider,
                )
                .await?,
            );
        }
        for pubkey in writable.into_iter() {
            let account_meta = TransactionAccountMeta::try_writable(
                pubkey,
                account_chain_snapshot_provider,
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
                TransactionAccountMeta::Writable { chain_snapshot, .. } => {
                    chain_snapshot.chain_state.is_inconsistent()
                }
                _ => false,
            })
            .map(|x| *x.pubkey())
            .collect()
    }

    pub fn writable_delegated_pubkeys(&self) -> Vec<Pubkey> {
        self.iter()
            .filter(|x| match x {
                TransactionAccountMeta::Writable { chain_snapshot, .. } => {
                    chain_snapshot.chain_state.is_delegated()
                }
                _ => false,
            })
            .map(|x| *x.pubkey())
            .collect()
    }

    pub fn writable_undelegated_non_payer_pubkeys(&self) -> Vec<Pubkey> {
        self.iter()
            .filter(|x| match x {
                TransactionAccountMeta::Writable {
                    is_payer,
                    chain_snapshot,
                    ..
                } => !chain_snapshot.chain_state.is_delegated() && !is_payer,
                _ => false,
            })
            .map(|x| *x.pubkey())
            .collect()
    }

    pub fn writable_new_pubkeys(&self) -> Vec<Pubkey> {
        self.iter()
            .filter(|x| match x {
                TransactionAccountMeta::Writable { chain_snapshot, .. } => {
                    chain_snapshot.chain_state.is_new()
                }
                _ => false,
            })
            .map(|x| *x.pubkey())
            .collect()
    }
}

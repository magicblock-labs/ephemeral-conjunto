use conjunto_core::{
    delegation_record_parser::DelegationRecordParser, AccountProvider,
};
use conjunto_lockbox::{
    account_chain_snapshot_provider::AccountChainSnapshotProvider,
    account_chain_snapshot_shared::AccountChainSnapshotShared,
};
use futures_util::future::{try_join, try_join_all, TryFutureExt};
use serde::{Deserialize, Serialize};
use solana_sdk::{clock::Slot, pubkey::Pubkey};

use crate::{
    errors::TranswiseResult,
    transaction_accounts_holder::TransactionAccountsHolder,
};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct TransactionAccountsSnapshot {
    pub readonly: Vec<AccountChainSnapshotShared>,
    pub writable: Vec<AccountChainSnapshotShared>,
    pub payer: Pubkey,
}

impl TransactionAccountsSnapshot {
    pub async fn from_accounts_holder<
        T: AccountProvider,
        V: DelegationRecordParser,
    >(
        holder: &TransactionAccountsHolder,
        account_chain_snapshot_provider: &AccountChainSnapshotProvider<T, V>,
        min_context_slot: Option<Slot>,
    ) -> TranswiseResult<Self> {
        // Fully parallelize snapshot fetching using join(s)
        let (readonly, writable) = try_join(
            try_join_all(holder.readonly.iter().map(|pubkey| {
                account_chain_snapshot_provider
                    .try_fetch_chain_snapshot_of_pubkey(
                        pubkey,
                        min_context_slot,
                    )
                    .map_ok(AccountChainSnapshotShared::from)
            })),
            try_join_all(holder.writable.iter().map(|pubkey| {
                account_chain_snapshot_provider
                    .try_fetch_chain_snapshot_of_pubkey(
                        pubkey,
                        min_context_slot,
                    )
                    .map_ok(AccountChainSnapshotShared::from)
            })),
        )
        .await?;
        Ok(Self {
            readonly,
            writable,
            payer: holder.payer,
        })
    }

    pub fn writable_undelegated_pubkeys(&self) -> Vec<Pubkey> {
        self.writable
            .iter()
            .filter(|chain_snapshot| {
                chain_snapshot.chain_state.is_undelegated()
            })
            .map(|chain_snapshot| chain_snapshot.pubkey)
            .collect()
    }

    pub fn writable_delegated_pubkeys(&self) -> Vec<Pubkey> {
        self.writable
            .iter()
            .filter(|chain_snapshot| chain_snapshot.chain_state.is_delegated())
            .map(|chain_snapshot| chain_snapshot.pubkey)
            .collect()
    }
}

use conjunto_core::{
    AccountProvider, DelegationRecord, DelegationRecordParser,
};
use conjunto_providers::{
    rpc_account_provider::RpcAccountProvider,
    rpc_provider_config::RpcProviderConfig,
};
use dlp::pda;
use serde::{Deserialize, Serialize};
use solana_sdk::{account::Account, clock::Slot, pubkey::Pubkey};

use crate::{
    accounts::predicates::is_owned_by_delegation_program,
    delegation_account::{DelegationAccount, DelegationRecordParserImpl},
    errors::{LockboxError, LockboxResult},
    AccountChainState, LockConfig,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AccountChainSnapshot {
    pub at_slot: Slot,
    pub chain_state: AccountChainState,
}

pub struct AccountChainSnapshotProvider<
    T: AccountProvider,
    U: DelegationRecordParser,
> {
    account_provider: T,
    delegation_record_parser: U,
}

impl<T: AccountProvider, U: DelegationRecordParser>
    AccountChainSnapshotProvider<T, U>
{
    pub fn new(
        config: RpcProviderConfig,
    ) -> AccountChainSnapshotProvider<
        RpcAccountProvider,
        DelegationRecordParserImpl,
    > {
        let rpc_account_provider = RpcAccountProvider::new(config);
        let delegation_record_parser = DelegationRecordParserImpl;
        AccountChainSnapshotProvider::with_provider_and_parser(
            rpc_account_provider,
            delegation_record_parser,
        )
    }

    pub fn new_with_parser(
        config: RpcProviderConfig,
        delegation_record_parser: U,
    ) -> AccountChainSnapshotProvider<RpcAccountProvider, U> {
        let rpc_account_provider = RpcAccountProvider::new(config);
        AccountChainSnapshotProvider::with_provider_and_parser(
            rpc_account_provider,
            delegation_record_parser,
        )
    }

    pub fn with_provider_and_parser(
        account_provider: T,
        delegation_record_parser: U,
    ) -> Self {
        Self {
            account_provider,
            delegation_record_parser,
        }
    }

    pub async fn try_fetch_chain_snapshot_of_pubkey(
        &self,
        pubkey: &Pubkey,
    ) -> LockboxResult<AccountChainSnapshot> {
        let delegation_pda = pda::delegation_record_pda_from_pubkey(pubkey);
        // Fetch the current chain state for revelant accounts (all at once)
        let (at_slot, mut fetched_accounts) = self
            .account_provider
            .get_multiple_accounts(&[delegation_pda, *pubkey])
            .await?;
        // Parse the result into an AccountChainState
        self.try_parse_chain_state_of_fetched_accounts(
            pubkey,
            delegation_pda,
            &mut fetched_accounts,
        )
        .map(|chain_state| AccountChainSnapshot {
            at_slot,
            chain_state,
        })
    }

    fn try_parse_chain_state_of_fetched_accounts(
        &self,
        pubkey: &Pubkey,
        delegation_pda: Pubkey,
        fetched_accounts: &mut Vec<Option<Account>>,
    ) -> LockboxResult<AccountChainState> {
        // If something went wrong in the fetch we stop, we should receive 2 accounts exactly every time
        if fetched_accounts.len() != 2 {
            return Err(LockboxError::InvalidFetch {
                fetched_pubkeys: vec![*pubkey, delegation_pda],
                fetched_accounts: fetched_accounts.clone(),
            });
        }
        // Check if the base account exists (it should always be account at index[1])
        let base_account = match fetched_accounts.remove(1) {
            Some(account) => account,
            None => return Ok(AccountChainState::NewAccount),
        };
        // Check if the base account is locked by the delegation program
        if !is_owned_by_delegation_program(&base_account) {
            return Ok(AccountChainState::Undelegated {
                account: base_account,
            });
        }
        // Verify the delegation account exists and is owned by the delegation program
        match DelegationAccount::try_from_fetched_account(
            fetched_accounts.remove(0),
            &self.delegation_record_parser,
        )? {
            DelegationAccount::Valid(DelegationRecord {
                commit_frequency,
                owner,
            }) => Ok(AccountChainState::Delegated {
                account: base_account,
                delegated_id: *pubkey,
                delegation_pda,
                config: LockConfig {
                    commit_frequency,
                    owner,
                },
            }),
            DelegationAccount::Invalid(inconsistencies) => {
                Ok(AccountChainState::Inconsistent {
                    account: base_account,
                    delegated_id: *pubkey,
                    delegation_pda,
                    inconsistencies,
                })
            }
        }
    }
}

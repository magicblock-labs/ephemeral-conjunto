use conjunto_core::{
    AccountProvider, DelegationRecord, DelegationRecordParser,
};
use conjunto_providers::{
    rpc_account_provider::RpcAccountProvider,
    rpc_provider_config::RpcProviderConfig,
};
use dlp::pda;
use serde::{Deserialize, Serialize};
use solana_sdk::{account::Account, pubkey::Pubkey};

use crate::{
    accounts::predicates::is_owned_by_delegation_program,
    delegation_account::{DelegationAccount, DelegationRecordParserImpl},
    errors::LockboxResult,
    LockConfig, LockInconsistency,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum AccountChainState {
    /// The account is not present on chain and thus not delegated either
    /// In this case we assume that this is an account that temporarily exists
    /// on the ephemeral validator and will not have to be undelegated.
    /// However in the short term we don't allow new accounts to be created inside
    /// the validator which means that we reject any transactions that attempt to do so
    NewAccount,
    /// The account was found on chain and is not delegated and therefore should
    /// not be used as writable on the ephemeral validator unless otherwise allowed
    /// via the `require_delegation=false` setting.
    Undelegated { account: Account },
    /// The account was found on chain in a proper delegated state which means we
    /// also found the related accounts like the buffer and delegation
    /// NOTE: commit records and state diff accountsk are not checked since an
    /// account is delegated and then used before the validator commits a state change.
    Delegated {
        account: Account,
        delegated_id: Pubkey,
        delegation_pda: Pubkey,
        config: LockConfig,
    },
    /// The account was found on chain and was partially delegated which means that
    /// it is owned by the delegation program but one or more of the related
    /// accounts were either not present or not owned by the delegation program
    Inconsistent {
        account: Account,
        delegated_id: Pubkey,
        delegation_pda: Pubkey,
        inconsistencies: Vec<LockInconsistency>,
    },
}

impl AccountChainState {
    pub fn is_new(&self) -> bool {
        matches!(self, AccountChainState::NewAccount)
    }

    pub fn is_delegated(&self) -> bool {
        matches!(self, AccountChainState::Delegated { .. })
    }

    pub fn is_undelegated(&self) -> bool {
        matches!(self, AccountChainState::Undelegated { .. })
    }

    pub fn is_inconsistent(&self) -> bool {
        matches!(self, AccountChainState::Inconsistent { .. })
    }

    pub fn lock_config(&self) -> Option<LockConfig> {
        match self {
            AccountChainState::Delegated { config, .. } => Some(config.clone()),
            _ => None,
        }
    }

    pub fn into_account(self) -> Option<Account> {
        match self {
            AccountChainState::NewAccount => None,
            AccountChainState::Undelegated { account } => Some(account),
            AccountChainState::Delegated { account, .. } => Some(account),
            AccountChainState::Inconsistent { account, .. } => Some(account),
        }
    }
}

pub struct AccountChainStateProvider<
    T: AccountProvider,
    U: DelegationRecordParser,
> {
    account_provider: T,
    delegation_record_parser: U,
}

impl<T: AccountProvider, U: DelegationRecordParser>
    AccountChainStateProvider<T, U>
{
    pub fn new(
        config: RpcProviderConfig,
    ) -> AccountChainStateProvider<RpcAccountProvider, DelegationRecordParserImpl>
    {
        let rpc_account_provider = RpcAccountProvider::new(config);
        let delegation_record_parser = DelegationRecordParserImpl;
        AccountChainStateProvider::with_provider_and_parser(
            rpc_account_provider,
            delegation_record_parser,
        )
    }

    pub fn new_with_parser(
        config: RpcProviderConfig,
        delegation_record_parser: U,
    ) -> AccountChainStateProvider<RpcAccountProvider, U> {
        let rpc_account_provider = RpcAccountProvider::new(config);
        AccountChainStateProvider::with_provider_and_parser(
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

    pub async fn try_fetch_chain_state_of_pubkey(
        &self,
        pubkey: &Pubkey,
    ) -> LockboxResult<AccountChainState> {
        // NOTE: this could be perf optimized by using get_multiple_accounts in one request
        // instead: https://docs.rs/solana-rpc-client/1.18.12/solana_rpc_client/nonblocking/rpc_client/struct.RpcClient.html#method.get_multiple_accounts
        // However that method returns one ClientResult, meaning if any account is not found it
        // we don't know which ones were there and which ones weren't and thus couldn't provide as
        // detailed information about the inconsistencies as we are now.

        // 1. Make sure the delegated account exists at all
        let account = match self.account_provider.get_account(pubkey).await? {
            Some(acc) => acc,
            None => {
                return Ok(AccountChainState::NewAccount);
            }
        };

        // 2. Check that it is locked by the delegation program
        if !is_owned_by_delegation_program(&account) {
            return Ok(AccountChainState::Undelegated { account });
        }

        // 3. Verify the delegation account exists and is owned by the delegation program
        let delegation_pda = pda::delegation_record_pda_from_pubkey(pubkey);
        use DelegationAccount::*;
        match DelegationAccount::try_from_pda_pubkey(
            &delegation_pda,
            &self.account_provider,
            &self.delegation_record_parser,
        )
        .await?
        {
            Valid(DelegationRecord {
                commit_frequency,
                owner,
            }) => Ok(AccountChainState::Delegated {
                account,
                delegated_id: *pubkey,
                delegation_pda,
                config: LockConfig {
                    commit_frequency,
                    owner,
                },
            }),
            Invalid(inconsistencies) => Ok(AccountChainState::Inconsistent {
                account,
                delegated_id: *pubkey,
                delegation_pda,
                inconsistencies,
            }),
        }
    }
}

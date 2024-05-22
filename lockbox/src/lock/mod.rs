use conjunto_addresses::pda;
use conjunto_core::{
    AccountProvider, CommitFrequency, DelegationRecord, DelegationRecordParser,
};
use conjunto_providers::{
    rpc_account_provider::RpcAccountProvider,
    rpc_provider_config::RpcProviderConfig,
};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::{
    accounts::predicates::is_owned_by_delegation_program,
    delegation_account::{DelegationAccount, DelegationRecordParserImpl},
    errors::LockboxResult,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct LockConfig {
    pub commit_frequency: CommitFrequency,
}

impl Default for LockConfig {
    fn default() -> Self {
        Self {
            commit_frequency: CommitFrequency::Millis(60_000),
        }
    }
}

impl From<DelegationRecord> for LockConfig {
    fn from(record: DelegationRecord) -> Self {
        Self {
            commit_frequency: record.commit_frequency,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LockInconsistency {
    DelegationAccountNotFound,
    BufferAccountInvalidOwner,
    DelegationAccountInvalidOwner,
    DelegationRecordAccountDataInvalid(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum AccountLockState {
    /// The account is not present on chain and thus not locked either
    /// In this case we assume that this is an account that temporarily exists
    /// on the ephemeral validator and will not have to be undelegated.
    /// However in the short term we don't allow new accounts to be created inside
    /// the validator which means that we reject any transactions that attempt to do
    /// that
    NewAccount,
    /// The account was found on chain and is not locked and therefore should
    /// not be used as writable on the ephemeral validator
    Unlocked,
    /// The account was found on chain in a proper locked state which means we
    /// also found the related accounts like the buffer and delegation
    /// NOTE: commit records and state diff accountsk are not checked since an
    /// account is delegated and then used before the validator commits a state change.
    Locked {
        delegated_id: Pubkey,
        delegation_pda: Pubkey,
        config: LockConfig,
    },
    /// The account was found on chain and was partially locked which means that
    /// it is owned by the delegation program but one or more of the related
    /// accounts were either not present or not owned by the delegation program
    Inconsistent {
        delegated_id: Pubkey,
        delegation_pda: Pubkey,
        inconsistencies: Vec<LockInconsistency>,
    },
}

impl AccountLockState {
    pub fn is_new(&self) -> bool {
        matches!(self, AccountLockState::NewAccount)
    }

    pub fn is_locked(&self) -> bool {
        matches!(self, AccountLockState::Locked { .. })
    }

    pub fn is_unlocked(&self) -> bool {
        matches!(self, AccountLockState::Unlocked)
    }

    pub fn is_inconsistent(&self) -> bool {
        matches!(self, AccountLockState::Inconsistent { .. })
    }

    pub fn config(&self) -> Option<&LockConfig> {
        match self {
            AccountLockState::Locked { config, .. } => Some(config),
            _ => None,
        }
    }
}

pub struct AccountLockStateProvider<
    T: AccountProvider,
    U: DelegationRecordParser,
> {
    account_provider: T,
    delegation_record_parser: U,
}

impl<T: AccountProvider, U: DelegationRecordParser>
    AccountLockStateProvider<T, U>
{
    pub fn new(
        config: RpcProviderConfig,
    ) -> AccountLockStateProvider<RpcAccountProvider, DelegationRecordParserImpl>
    {
        let rpc_account_provider = RpcAccountProvider::new(config);
        let delegation_record_parser = DelegationRecordParserImpl;
        AccountLockStateProvider::with_provider_and_parser(
            rpc_account_provider,
            delegation_record_parser,
        )
    }

    pub fn new_with_parser(
        config: RpcProviderConfig,
        delegation_record_parser: U,
    ) -> AccountLockStateProvider<RpcAccountProvider, U> {
        let rpc_account_provider = RpcAccountProvider::new(config);
        AccountLockStateProvider::with_provider_and_parser(
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

    pub async fn try_lockstate_of_pubkey(
        &self,
        pubkey: &Pubkey,
    ) -> LockboxResult<AccountLockState> {
        // NOTE: this could be perf optimized by using get_multiple_accounts in one request
        // instead: https://docs.rs/solana-rpc-client/1.18.12/solana_rpc_client/nonblocking/rpc_client/struct.RpcClient.html#method.get_multiple_accounts
        // However that method returns one ClientResult, meaning if any account is not found it
        // we don't know which ones were there and which ones weren't and thus couldn't provide as
        // detailed information about the inconsistencies as we are now.

        let delegation_pda = pda::delegation_pda_from_pubkey(pubkey);

        // 1. Make sure the delegate account exists at all
        let account = match self.account_provider.get_account(pubkey).await? {
            Some(acc) => acc,
            None => {
                return Ok(AccountLockState::NewAccount);
            }
        };

        // 2. Check that it is locked by the delegation program
        if !is_owned_by_delegation_program(&account) {
            return Ok(AccountLockState::Unlocked);
        }

        // 3. Verify the delegation account exists and is owned by the delegation program
        use DelegationAccount::*;
        match DelegationAccount::try_from_pda_pubkey(
            &delegation_pda,
            &self.account_provider,
            &self.delegation_record_parser,
        )
        .await?
        {
            Valid(DelegationRecord { commit_frequency }) => {
                Ok(AccountLockState::Locked {
                    delegated_id: *pubkey,
                    delegation_pda,
                    config: LockConfig { commit_frequency },
                })
            }
            Invalid(inconsistencies) => Ok(AccountLockState::Inconsistent {
                delegated_id: *pubkey,
                delegation_pda,
                inconsistencies,
            }),
        }
    }
}

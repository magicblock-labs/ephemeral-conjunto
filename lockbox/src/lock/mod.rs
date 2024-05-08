use conjunto_addresses::pda;
use solana_sdk::pubkey::Pubkey;

use crate::{
    accounts::{
        predicates::is_owned_by_delegation_program,
        rpc_account_provider::{RpcAccountProvider, RpcAccountProviderConfig},
        AccountProvider,
    },
    errors::LockboxResult,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LockInconsistency {
    DelegateAccountNotFound,
    BufferAccountNotFound,
    DelegationAccountNotFound,
    BufferAccountInvalidOwner,
    DelegationAccountInvalidOwner,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AccountLockState {
    Unlocked,
    // TODO(thlorenz): what about state diff and commit record
    // - are they expected at the PDA addresses?
    // - are they optional?
    // - should we indicate if they were found?
    // - if so what predicates do they need to match?
    Locked {
        delegated_id: Pubkey,
        buffer_pda: Pubkey,
        delegation_pda: Pubkey,
    },
    Inconsistent {
        delegated_id: Pubkey,
        buffer_pda: Pubkey,
        delegation_pda: Pubkey,
        inconsistencies: Vec<LockInconsistency>,
    },
}

pub struct AccountLockStateProvider<T: AccountProvider> {
    account_provider: T,
}

impl<T: AccountProvider> AccountLockStateProvider<T> {
    pub fn new(
        config: RpcAccountProviderConfig,
    ) -> AccountLockStateProvider<RpcAccountProvider> {
        let rpc_account_provider = RpcAccountProvider::new(config);
        AccountLockStateProvider::with_provider(rpc_account_provider)
    }

    pub fn with_provider(account_provider: T) -> Self {
        Self { account_provider }
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

        let buffer_pda = pda::buffer_pda_from_pubkey(pubkey);
        let delegation_pda = pda::delegation_pda_from_pubkey(pubkey);

        // 1. Make sure the delegate account exists at all
        let account = match self.account_provider.get_account(pubkey).await? {
            Some(acc) => acc,
            None => {
                return Ok(AccountLockState::Inconsistent {
                    delegated_id: *pubkey,
                    buffer_pda,
                    delegation_pda,
                    inconsistencies: vec![
                        LockInconsistency::DelegateAccountNotFound,
                    ],
                });
            }
        };

        // 2. Check that it is locked by the delegation program
        if !is_owned_by_delegation_program(&account) {
            return Ok(AccountLockState::Unlocked);
        }

        let mut inconsistencies = Vec::<LockInconsistency>::new();

        // 3. Verify the buffer account exists and is owned by the delegation program
        if let Some(xs) = self.verify_buffer_account(&buffer_pda).await? {
            inconsistencies.extend(xs);
        }

        // 4. Verify the delegation account exists and is owned by the delegation program
        if let Some(xs) =
            self.verify_delegation_account(&delegation_pda).await?
        {
            inconsistencies.extend(xs);
        }

        if inconsistencies.is_empty() {
            Ok(AccountLockState::Locked {
                delegated_id: *pubkey,
                buffer_pda,
                delegation_pda,
            })
        } else {
            Ok(AccountLockState::Inconsistent {
                delegated_id: *pubkey,
                buffer_pda,
                delegation_pda,
                inconsistencies,
            })
        }
    }

    async fn verify_buffer_account(
        &self,
        buffer_pda: &Pubkey,
    ) -> LockboxResult<Option<Vec<LockInconsistency>>> {
        let buffer_account =
            match self.account_provider.get_account(buffer_pda).await? {
                None => {
                    return Ok(Some(vec![
                        LockInconsistency::BufferAccountNotFound,
                    ]))
                }
                Some(acc) => acc,
            };

        if !is_owned_by_delegation_program(&buffer_account) {
            Ok(Some(vec![LockInconsistency::BufferAccountInvalidOwner]))
        } else {
            Ok(None)
        }
    }

    async fn verify_delegation_account(
        &self,
        delegation_pda: &Pubkey,
    ) -> LockboxResult<Option<Vec<LockInconsistency>>> {
        let delegation_account =
            match self.account_provider.get_account(delegation_pda).await? {
                None => {
                    return Ok(Some(vec![
                        LockInconsistency::DelegationAccountNotFound,
                    ]))
                }
                Some(acc) => acc,
            };

        if !is_owned_by_delegation_program(&delegation_account) {
            Ok(Some(vec![LockInconsistency::DelegationAccountInvalidOwner]))
        } else {
            Ok(None)
        }
    }
}

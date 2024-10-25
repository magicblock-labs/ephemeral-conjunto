use conjunto_core::{
    delegation_inconsistency::DelegationInconsistency,
    delegation_record_parser::DelegationRecordParser, AccountProvider,
};
use dlp::{consts::DELEGATION_PROGRAM_ID, pda};
use solana_sdk::{account::Account, pubkey::Pubkey, system_program};

use crate::{
    account_chain_snapshot::AccountChainSnapshot,
    account_chain_state::AccountChainState,
    errors::{LockboxError, LockboxResult},
};

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
    pub fn new(account_provider: T, delegation_record_parser: U) -> Self {
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
            .get_multiple_accounts(&[*pubkey, delegation_pda])
            .await?;
        // If something went wrong in the fetch we stop, we should receive 2 accounts exactly every time
        if fetched_accounts.len() != 2 {
            return Err(LockboxError::InvalidFetch {
                fetched_pubkeys: vec![*pubkey, delegation_pda],
                fetched_accounts,
            });
        }
        // Extract the accounts we just fetched
        let account = fetched_accounts.swap_remove(0);
        let delegation_record_account = fetched_accounts.swap_remove(0);
        // Parse the result into an AccountChainState
        let chain_state = self.try_into_chain_state_from_fetched_accounts(
            pubkey,
            account,
            delegation_record_account,
        );
        // Build the AccountChainSnapshot
        Ok(AccountChainSnapshot {
            pubkey: *pubkey,
            at_slot,
            chain_state,
        })
    }

    fn try_into_chain_state_from_fetched_accounts(
        &self,
        address: &Pubkey,
        account: Option<Account>,
        delegation_record_account: Option<Account>,
    ) -> AccountChainState {
        // Check if the base account exists
        let account = match account {
            None => {
                return AccountChainState::FeePayer {
                    lamports: 0,
                    owner: system_program::ID,
                }
            }
            Some(account) => account,
        };
        // Check if the base account is locked by the delegation program
        if !is_owned_by_delegation_program(&account) {
            // If the account is not locked, does not have any data, is on-curve and is system program owned, it's a fee-payer
            if account.data.is_empty()
                && system_program::check_id(&account.owner)
                && address.is_on_curve()
            {
                return AccountChainState::FeePayer {
                    lamports: account.lamports,
                    owner: account.owner,
                };
            }
            // If the account is no locked and does not meet the criteria above, it's undelegated
            else {
                return AccountChainState::Undelegated {
                    account,
                    delegation_inconsistency:
                        DelegationInconsistency::AccountInvalidOwner,
                };
            }
        }
        // Check if the delegation record exists
        let delegation_record_account = match delegation_record_account {
            None => {
                return AccountChainState::Undelegated {
                    account,
                    delegation_inconsistency:
                        DelegationInconsistency::DelegationRecordNotFound,
                }
            }
            Some(account) => account,
        };
        // Check if the delegation record is owned by the delegation program
        if !is_owned_by_delegation_program(&delegation_record_account) {
            return AccountChainState::Undelegated {
                account,
                delegation_inconsistency:
                    DelegationInconsistency::DelegationRecordInvalidOwner,
            };
        }
        // Try to parse the delegation record's data
        match self
            .delegation_record_parser
            .try_parse(&delegation_record_account.data)
        {
            Err(err) => AccountChainState::Undelegated {
                account,
                delegation_inconsistency:
                    DelegationInconsistency::DelegationRecordDataInvalid(
                        err.to_string(),
                    ),
            },
            Ok(delegation_record) => AccountChainState::Delegated {
                account,
                delegation_record,
            },
        }
    }
}

fn is_owned_by_delegation_program(account: &Account) -> bool {
    account.owner == DELEGATION_PROGRAM_ID
}

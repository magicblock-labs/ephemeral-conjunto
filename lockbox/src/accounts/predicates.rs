use dlp::consts::DELEGATION_PROGRAM_ID;
use solana_sdk::account::Account;

pub fn is_owned_by_delegation_program(account: &Account) -> bool {
    account.owner == DELEGATION_PROGRAM_ID
}

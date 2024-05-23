use std::str::FromStr;

use solana_sdk::{account::Account, pubkey::Pubkey, system_program};

/// The bytes of the program ID of the delegation program
pub const DELEGATION_PROGRAM_ARRAY: [u8; 32] = [
    181, 183, 0, 225, 242, 87, 58, 192, 204, 6, 34, 1, 52, 74, 207, 151, 184,
    53, 6, 235, 140, 229, 25, 152, 204, 98, 126, 24, 147, 128, 167, 62,
];

/// The program ID of the delegation program
pub const DELEGATION_PROGRAM_ID: Pubkey =
    Pubkey::new_from_array(DELEGATION_PROGRAM_ARRAY);

pub fn account_owned_by_delegation_program() -> Account {
    Account {
        owner: DELEGATION_PROGRAM_ID,
        ..Account::default()
    }
}

pub fn account_owned_by_system_program() -> Account {
    Account {
        owner: system_program::id(),
        ..Account::default()
    }
}

pub fn program_account() -> Account {
    Account {
        executable: true,
        ..Account::default()
    }
}

pub fn delegated_account_ids() -> (Pubkey, Pubkey) {
    let delegated_addr = "8k2V7EzQtNg38Gi9HK5ZtQYp1YpGKNGrMcuGa737gZX4";
    let delegated_id = Pubkey::from_str(delegated_addr).unwrap();

    let delegation_addr = "CkieZJmrj6dLhwteG69LSutpWwRHiDJY9S8ua7xJ7CRW";
    let delegation_pda = Pubkey::from_str(delegation_addr).unwrap();

    (delegated_id, delegation_pda)
}

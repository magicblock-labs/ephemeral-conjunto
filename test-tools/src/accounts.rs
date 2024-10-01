use solana_sdk::{account::Account, pubkey, pubkey::Pubkey, system_program};

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

pub fn account_with_data() -> Account {
    Account {
        owner: Pubkey::new_unique(),
        data: vec![1, 2, 3, 4],
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
    let delegated_id = pubkey!("8k2V7EzQtNg38Gi9HK5ZtQYp1YpGKNGrMcuGa737gZX4");
    let delegation_pda =
        pubkey!("CkieZJmrj6dLhwteG69LSutpWwRHiDJY9S8ua7xJ7CRW");
    (delegated_id, delegation_pda)
}

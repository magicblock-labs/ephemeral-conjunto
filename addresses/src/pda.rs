use paste::paste;

use crate::consts::{COMMIT_RECORD, DELEGATION, STATE_DIFF};

// -----------------
// Seeds
// -----------------
macro_rules! seeds {
    ($prefix:ident, $bytes_const:expr) => {
        paste! {
            #[allow(clippy::needless_lifetimes)]
            pub fn [<$prefix _seeds>]<'a>(pda_id: &'a [u8]) -> [&'a [u8]; 2] {
                [$bytes_const, pda_id]
            }
            #[allow(clippy::needless_lifetimes)]
            pub fn [<$prefix _seeds_with_bump>]<'a>(pda_id: &'a [u8], bump: &'a [u8; 1]) -> [&'a [u8]; 3] {
                [$bytes_const, pda_id, bump]
            }
            #[allow(clippy::needless_lifetimes)]
            pub fn [<$prefix _seeds_from_pubkey>]<'a>(pda_id: &'a ::solana_sdk::pubkey::Pubkey) -> [&'a [u8]; 2] {
                [$bytes_const, pda_id.as_ref()]
            }
            #[allow(clippy::needless_lifetimes)]
            pub fn [<$prefix _seeds_with_bump_from_pubkey>]<'a>(
                pda_id: &'a ::solana_sdk::pubkey::Pubkey,
                bump: &'a [u8; 1],
            ) -> [&'a [u8]; 3] {
                [$bytes_const, pda_id.as_ref(), bump]
            }
        }
    };
}

// -----------------
// PDA
// -----------------
macro_rules! pda {
    ($prefix:ident) => {
        paste! {
            #[allow(clippy::needless_lifetimes)]
            pub fn [<$prefix _pda_with_bump>]<'a>(pda_id: &'a [u8]) -> (::solana_sdk::pubkey::Pubkey, u8) {
                let seeds = [<$prefix _seeds>](pda_id);
                ::solana_sdk::pubkey::Pubkey::find_program_address(
                    &seeds,
                    &crate::consts::DELEGATION_PROGRAM_ID
                )
            }
            #[allow(clippy::needless_lifetimes)]
            pub fn [<$prefix _pda>]<'a>(pda_id: &'a [u8]) -> ::solana_sdk::pubkey::Pubkey {
                [<$prefix _pda_with_bump>](pda_id).0
            }
            #[allow(clippy::needless_lifetimes)]
            pub fn [<$prefix _pda_with_bump_from_pubkey>]<'a>(pda_id: &'a ::solana_sdk::pubkey::Pubkey) -> (::solana_sdk::pubkey::Pubkey, u8) {
                let seeds = [<$prefix _seeds_from_pubkey>](pda_id);
                ::solana_sdk::pubkey::Pubkey::find_program_address(
                    &seeds,
                    &crate::consts::DELEGATION_PROGRAM_ID
                )
            }
            #[allow(clippy::needless_lifetimes)]
            pub fn [<$prefix _pda_from_pubkey>]<'a>(pda_id: &'a ::solana_sdk::pubkey::Pubkey) -> ::solana_sdk::pubkey::Pubkey {
                [<$prefix _pda_with_bump_from_pubkey>](pda_id).0
            }
        }
    };
}

seeds! { delegation, DELEGATION }
pda! { delegation }

seeds! { state_diff, STATE_DIFF }
pda! { state_diff }

seeds! { commit_record, COMMIT_RECORD }
pda! { commit_record }

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use solana_sdk::pubkey::Pubkey;

    use super::*;
    use crate::consts::{
        COMMIT_RECORD, DELEGATION, DELEGATION_PROGRAM_ID, STATE_DIFF,
    };

    // -----------------
    // Delegation Seeds
    // -----------------
    #[test]
    fn test_delegation_seeds() {
        let id = [1, 2, 3];
        let seeds = delegation_seeds(&id);
        assert_eq!(seeds, [DELEGATION, &id]);
    }

    #[test]
    fn test_delegation_seeds_with_bump() {
        let id = [1, 2, 3];
        let bump = [4];
        let seeds = delegation_seeds_with_bump(&id, &bump);
        assert_eq!(seeds, [DELEGATION, &id, &bump]);
    }

    #[test]
    fn test_delegation_seeds_from_pubkey() {
        let id = Pubkey::new_unique();
        let seeds = delegation_seeds_from_pubkey(&id);
        assert_eq!(seeds, [DELEGATION, id.as_ref()]);
    }

    #[test]
    fn test_delegation_seeds_with_bump_from_pubkey() {
        let id = Pubkey::new_unique();
        let bump = [4];
        let seeds = delegation_seeds_with_bump_from_pubkey(&id, &bump);
        assert_eq!(seeds, [DELEGATION, id.as_ref(), &bump]);
    }

    // -----------------
    // State Diff Seeds
    // -----------------
    #[test]
    fn test_state_diff_seeds() {
        let id = [1, 2, 3];
        let seeds = state_diff_seeds(&id);
        assert_eq!(seeds, [STATE_DIFF, &id]);
    }

    #[test]
    fn test_state_diff_seeds_with_bump() {
        let id = [1, 2, 3];
        let bump = [4];
        let seeds = state_diff_seeds_with_bump(&id, &bump);
        assert_eq!(seeds, [STATE_DIFF, &id, &bump]);
    }

    #[test]
    fn test_state_diff_seeds_from_pubkey() {
        let id = Pubkey::new_unique();
        let seeds = state_diff_seeds_from_pubkey(&id);
        assert_eq!(seeds, [STATE_DIFF, id.as_ref()]);
    }

    #[test]
    fn test_state_diff_seeds_with_bump_from_pubkey() {
        let id = Pubkey::new_unique();
        let bump = [4];
        let seeds = state_diff_seeds_with_bump_from_pubkey(&id, &bump);
        assert_eq!(seeds, [STATE_DIFF, id.as_ref(), &bump]);
    }

    // -----------------
    // Commit Record Seeds
    // -----------------
    #[test]
    fn test_commit_record_seeds() {
        let id = [1, 2, 3];
        let seeds = commit_record_seeds(&id);
        assert_eq!(seeds, [COMMIT_RECORD, &id]);
    }

    #[test]
    fn test_commit_record_seeds_with_bump() {
        let id = [1, 2, 3];
        let bump = [4];
        let seeds = commit_record_seeds_with_bump(&id, &bump);
        assert_eq!(seeds, [COMMIT_RECORD, &id, &bump]);
    }

    #[test]
    fn test_commit_record_seeds_from_pubkey() {
        let id = Pubkey::new_unique();
        let seeds = commit_record_seeds_from_pubkey(&id);
        assert_eq!(seeds, [COMMIT_RECORD, id.as_ref()]);
    }

    #[test]
    fn test_commit_record_seeds_with_bump_from_pubkey() {
        let id = Pubkey::new_unique();
        let bump = [4];
        let seeds = commit_record_seeds_with_bump_from_pubkey(&id, &bump);
        assert_eq!(seeds, [COMMIT_RECORD, id.as_ref(), &bump]);
    }

    // -----------------
    // Delegation PDA
    // -----------------
    #[test]
    fn test_delegation_pda() {
        let id = Pubkey::new_unique();
        let pda = delegation_pda(id.as_ref());
        let seeds = delegation_seeds(id.as_ref());
        let expected =
            Pubkey::find_program_address(&seeds, &DELEGATION_PROGRAM_ID).0;
        assert_eq!(pda, expected);
    }

    #[test]
    fn test_delegation_pda_with_bump() {
        let id = Pubkey::new_unique();
        let (pda, bump) = delegation_pda_with_bump(id.as_ref());
        let seeds = delegation_seeds(id.as_ref());
        let expected =
            Pubkey::find_program_address(&seeds, &DELEGATION_PROGRAM_ID);
        assert_eq!(pda, expected.0);
        assert_eq!(bump, expected.1);
    }

    #[test]
    fn test_delegation_pda_from_pubkey() {
        let id = Pubkey::new_unique();
        let pda = delegation_pda_from_pubkey(&id);
        let seeds = delegation_seeds_from_pubkey(&id);
        let expected =
            Pubkey::find_program_address(&seeds, &DELEGATION_PROGRAM_ID).0;
        assert_eq!(pda, expected);
    }

    #[test]
    fn test_delegation_pda_with_bump_from_pubkey() {
        let id = Pubkey::new_unique();
        let (pda, bump) = delegation_pda_with_bump_from_pubkey(&id);
        let seeds = delegation_seeds_from_pubkey(&id);
        let expected =
            Pubkey::find_program_address(&seeds, &DELEGATION_PROGRAM_ID);
        assert_eq!(pda, expected.0);
        assert_eq!(bump, expected.1);
    }

    // NOTE: left out remaining checks since they all are implemented via the same macro

    #[test]
    fn test_known_delegation() {
        let delegated_addr = "8k2V7EzQtNg38Gi9HK5ZtQYp1YpGKNGrMcuGa737gZX4";
        let delegated_id = Pubkey::from_str(delegated_addr).unwrap();

        let delegation_addr = "CkieZJmrj6dLhwteG69LSutpWwRHiDJY9S8ua7xJ7CRW";
        let delegation_id = Pubkey::from_str(delegation_addr).unwrap();

        let state_diff_addr = "BUrsNkRnqoWUJdGotRt1odFp2NH5b9tcciXzXNbNwBHr";
        let state_diff_id = Pubkey::from_str(state_diff_addr).unwrap();

        let commit_record_addr = "GiDjQqUKeKJwLH5kdbnCgFS2XPGAVjXo73JMoeVn3UZL";
        let commit_record_id = Pubkey::from_str(commit_record_addr).unwrap();

        let delegation_pda = delegation_pda_from_pubkey(&delegated_id);
        let state_diff_pda = state_diff_pda_from_pubkey(&delegated_id);
        let commit_record_pda = commit_record_pda_from_pubkey(&delegated_id);

        assert_eq!(delegation_pda, delegation_id);
        assert_eq!(state_diff_pda, state_diff_id);
        assert_eq!(commit_record_pda, commit_record_id);
    }
}
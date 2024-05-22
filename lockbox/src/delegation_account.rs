use conjunto_core::{
    errors::CoreResult, AccountProvider, CommitFrequency, DelegationRecord,
    DelegationRecordParser,
};
use solana_sdk::pubkey::Pubkey;

use crate::{
    accounts::predicates::is_owned_by_delegation_program,
    errors::LockboxResult, LockInconsistency,
};

pub struct DelegationRecordParserImpl;
impl DelegationRecordParser for DelegationRecordParserImpl {
    fn try_parse(&self, _data: &[u8]) -> CoreResult<DelegationRecord> {
        Ok(DelegationRecord {
            // TODO(thlorenz): parse data using delegation program structs
            commit_frequency: CommitFrequency::Millis(1_000),
        })
    }
}

pub enum DelegationAccount {
    Valid(DelegationRecord),
    Invalid(Vec<LockInconsistency>),
}

impl DelegationAccount {
    pub async fn try_from_pda_pubkey<
        T: AccountProvider,
        U: DelegationRecordParser,
    >(
        delegation_pda: &Pubkey,
        account_provider: &T,
        delegation_record_parser: &U,
    ) -> LockboxResult<DelegationAccount> {
        let delegation_account =
            match account_provider.get_account(delegation_pda).await? {
                None => {
                    return Ok(DelegationAccount::Invalid(vec![
                        LockInconsistency::DelegationAccountNotFound,
                    ]))
                }
                Some(acc) => acc,
            };

        let mut inconsistencies = vec![];
        if !is_owned_by_delegation_program(&delegation_account) {
            inconsistencies
                .push(LockInconsistency::DelegationAccountInvalidOwner);
        }
        match delegation_record_parser.try_parse(&delegation_account.data) {
            Ok(delegation_record) => {
                if inconsistencies.is_empty() {
                    Ok(DelegationAccount::Valid(delegation_record))
                } else {
                    Ok(DelegationAccount::Invalid(inconsistencies))
                }
            }
            Err(err) => {
                inconsistencies.push(
                    LockInconsistency::DelegationRecordAccountDataInvalid(
                        err.to_string(),
                    ),
                );
                Ok(DelegationAccount::Invalid(inconsistencies))
            }
        }
    }
}

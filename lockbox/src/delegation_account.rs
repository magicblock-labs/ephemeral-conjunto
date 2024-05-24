use conjunto_core::{
    errors::{CoreError, CoreResult},
    AccountProvider, CommitFrequency, DelegationRecord, DelegationRecordParser,
};
use solana_sdk::pubkey::Pubkey;

use crate::{
    accounts::predicates::is_owned_by_delegation_program,
    errors::LockboxResult, LockInconsistency,
};

pub struct DelegationRecordParserImpl;
impl DelegationRecordParser for DelegationRecordParserImpl {
    fn try_parse(&self, data: &[u8]) -> CoreResult<DelegationRecord> {
        parse_delegation_record(data)
    }
}

fn parse_delegation_record(data: &[u8]) -> CoreResult<DelegationRecord> {
    // bytemuck fails with TargetAlignmentGreaterAndInputNotAligned when the data isn't
    // properly aligned. This happens sporadically depending on how the data was stored, i.e.
    // only in debug mode and is fine in release mode or if we add unrelated code before the call.
    // The below forces the data to be aligned since vecs are always aligned.
    // NOTE: I didn't find 100% confirmation that vecs are always correctly aligned, but
    // the issue I encountered was fixed by this change.
    // TODO(thlorenz): with this fix we copy data and should revisit this to avoid that
    let data = data.to_vec();
    let result =
        bytemuck::try_from_bytes::<dlp::state::DelegationRecord>(&data[8..])
            .map_err(|err| {
                CoreError::FailedToParseDelegationRecord(format!(
                    "Failed to deserialize DelegationRecord: {}",
                    err
                ))
            });

    let state = result.unwrap();
    Ok(DelegationRecord {
        owner: state.owner,
        commit_frequency: CommitFrequency::Millis(state.commit_frequency_ms),
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegation_record_parser() {
        // NOTE: from delegation-program/tests/fixtures/accounts.rs
        let delegation_record_account_data: [u8; 88] = [
            100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 43, 85, 175,
            207, 195, 148, 154, 129, 218, 62, 110, 177, 81, 112, 72, 172, 141,
            157, 3, 211, 24, 26, 191, 79, 101, 191, 48, 19, 105, 181, 70, 132,
            0, 0, 0, 0, 0, 0, 0, 0, 224, 147, 4, 0, 0, 0, 0, 0,
        ];
        let parser = DelegationRecordParserImpl;
        let record = parser.try_parse(&delegation_record_account_data).unwrap();
        assert_eq!(
            record,
            DelegationRecord {
                owner: <Pubkey as std::str::FromStr>::from_str(
                    "3vAK9JQiDsKoQNwmcfeEng4Cnv22pYuj1ASfso7U4ukF"
                )
                .unwrap(),
                commit_frequency: CommitFrequency::Millis(300_000),
            }
        );
    }
}

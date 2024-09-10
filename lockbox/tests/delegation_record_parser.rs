use conjunto_core::{
    delegation_record::{CommitFrequency, DelegationRecord},
    delegation_record_parser::DelegationRecordParser,
};
use conjunto_lockbox::delegation_record_parser_impl::DelegationRecordParserImpl;
use solana_sdk::pubkey;

#[test]
fn test_delegation_record_parser() {
    // NOTE: from delegation-program/tests/fixtures/accounts.rs
    let delegation_record_account_data: [u8; 88] = [
        100, 0, 0, 0, 0, 0, 0, 0, 168, 101, 177, 208, 38, 36, 83, 217, 138,
        159, 42, 183, 213, 78, 109, 216, 63, 161, 136, 242, 27, 0, 117, 150,
        140, 96, 0, 92, 107, 81, 86, 247, 43, 85, 175, 207, 195, 148, 154, 129,
        218, 62, 110, 177, 81, 112, 72, 172, 141, 157, 3, 211, 24, 26, 191, 79,
        101, 191, 48, 19, 105, 181, 70, 132, 4, 0, 0, 0, 0, 0, 0, 0, 48, 117,
        0, 0, 0, 0, 0, 0,
    ];
    let parser = DelegationRecordParserImpl;
    let record = parser.try_parse(&delegation_record_account_data).unwrap();
    assert_eq!(
        record,
        DelegationRecord {
            authority: pubkey!("CLMS5guJDje8BA9tQdd1wXmGmPx5S32yhGztw4xytAYN"),
            owner: pubkey!("3vAK9JQiDsKoQNwmcfeEng4Cnv22pYuj1ASfso7U4ukF"),
            delegation_slot: 4,
            commit_frequency: CommitFrequency::Millis(30_000),
        }
    );
}

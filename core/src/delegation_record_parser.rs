use crate::{delegation_record::DelegationRecord, errors::CoreResult};

pub trait DelegationRecordParser {
    fn try_parse(&self, data: &[u8]) -> CoreResult<DelegationRecord>;
}

use std::collections::HashMap;

use async_trait::async_trait;
use conjunto_core::{errors::CoreResult, SignatureStatusProvider};
use solana_sdk::{signature::Signature, transaction};

#[derive(Default)]
pub struct SignatureStatusProviderStub {
    pub signature_status: HashMap<Signature, transaction::Result<()>>,
}

impl SignatureStatusProviderStub {
    pub fn add(
        &mut self,
        signature: Signature,
        status: transaction::Result<()>,
    ) {
        self.signature_status.insert(signature, status);
    }
    pub fn add_ok(&mut self, signature: Signature) {
        self.signature_status
            .insert(signature, transaction::Result::Ok(()));
    }
}

#[async_trait]
impl SignatureStatusProvider for SignatureStatusProviderStub {
    async fn get_signature_status(
        &self,
        signature: &Signature,
    ) -> CoreResult<Option<transaction::Result<()>>> {
        Ok(self.signature_status.get(signature).cloned())
    }
}

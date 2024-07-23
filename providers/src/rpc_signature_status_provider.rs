use async_trait::async_trait;
use conjunto_core::{errors::CoreResult, SignatureStatusProvider};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_rpc_client_api::{client_error::ErrorKind, request::RpcError};
use solana_sdk::{
    commitment_config::CommitmentConfig, signature::Signature, transaction,
};

use crate::rpc_provider_config::RpcProviderConfig;

pub struct RpcSignatureStatusProvider {
    rpc_client: RpcClient,
}

impl RpcSignatureStatusProvider {
    pub fn new(config: RpcProviderConfig) -> Self {
        let rpc_client = RpcClient::new_with_commitment(
            config.cluster().url().to_string(),
            CommitmentConfig {
                commitment: config.commitment().unwrap_or_default(),
            },
        );
        Self { rpc_client }
    }
}

#[async_trait]
impl SignatureStatusProvider for RpcSignatureStatusProvider {
    async fn get_signature_status(
        &self,
        signature: &Signature,
    ) -> CoreResult<Option<transaction::Result<()>>> {
        let status = match self.rpc_client.get_signature_status(signature).await
        {
            Ok(status) => status,
            Err(err) => match err.kind() {
                ErrorKind::RpcError(RpcError::ForUser(msg)) => {
                    // TODO: what error do we actually get here?
                    eprintln!(
                        "SignatureStatusProvider RpcError::ForUser: {}",
                        msg
                    );
                    if msg.contains("SignatureNotFound") {
                        None
                    } else {
                        return Err(err.into());
                    }
                }
                _ => {
                    return Err(err.into());
                }
            },
        };
        Ok(status)
    }
}

use conjunto_transwise::trans_account_meta::Endpoint;
use jsonrpsee::{
    core::RpcResult,
    core::{client::ClientT, RegisterMethodError},
    RpcModule,
};
use log::*;
use solana_rpc_client_api::config::RpcSendTransactionConfig;
use solana_sdk::transaction::VersionedTransaction;
use solana_transaction_status::UiTransactionEncoding;

use super::DirectorRpc;
use crate::{
    decoders::decode_and_deserialize,
    rpc::params::SendTransactionParams,
    utils::{
        invalid_params, server_error, server_error_with_data, ServerErrorCode,
    },
};

pub fn register_guide_methods(
    module: &mut RpcModule<DirectorRpc>,
) -> Result<(), RegisterMethodError> {
    module.register_async_method(
        "sendTransaction",
        |params, rpc| async move {
            debug!("send_transaction rpc request received {:#?}", params);
            let SendTransactionParams(data, config) =
                params.parse::<SendTransactionParams>()?;

            rpc.send_transaction(data, config).await
        },
    )?;

    Ok(())
}

impl DirectorRpc {
    async fn send_transaction(
        &self,
        data: String,
        config: Option<RpcSendTransactionConfig>,
    ) -> RpcResult<String> {
        debug!("send_transaction rpc request received");
        // 1. Deserialize Transaction
        let RpcSendTransactionConfig {
            skip_preflight: _,
            preflight_commitment: _,
            encoding,
            max_retries: _,
            min_context_slot: _,
        } = config.unwrap_or_default();

        let tx_encoding = encoding.unwrap_or(UiTransactionEncoding::Base58);

        let binary_encoding = tx_encoding.into_binary_encoding().ok_or_else(|| {
                invalid_params(format!(
                    "unsupported encoding: {tx_encoding}. Supported encodings: base58, base64"
                ))
            })?;
        let (_, versioned_tx) = decode_and_deserialize::<VersionedTransaction>(
            &data,
            binary_encoding,
        )?;

        // 2. Determine Endpoint to be used for this Transaction
        let endpoint = match self
            .transwise
            .guide_versioned_transaction(&versioned_tx)
            .await
        {
            Ok(endpoint) => endpoint,
            Err(err) => {
                return Err(server_error(
                    format!("error: {err}"),
                    ServerErrorCode::FailedToFetchEndpointInformation,
                ));
            }
        };
        // 3. Route transaction accordingly
        info!("endpoint: {:#?}", endpoint);

        use Endpoint::*;
        match &endpoint {
            Chain(_) => Ok(self
                .rpc_chain_client
                .request("sendTransaction", SendTransactionParams(data, config))
                .await
                .map_err(|err| {
                    server_error_with_data(
                        format!("Failed to forward to on-chain RPC: {err:?}"),
                        ServerErrorCode::RpcClientError,
                        endpoint,
                    )
                })?),
            Ephemeral(_) => Ok(self
                .rpc_ephem_client
                .request("sendTransaction", SendTransactionParams(data, config))
                .await
                .map_err(|err| {
                    server_error_with_data(
                        format!("Failed to forward to ephemeral RPC: {err:?}"),
                        ServerErrorCode::RpcClientError,
                        endpoint,
                    )
                })?),
            Unroutable {
                account_metas: _,
                reason: _,
            } => Err(server_error_with_data(
                "Transaction is unroutable".to_string(),
                ServerErrorCode::TransactionUnroutable,
                endpoint,
            )),
        }
    }
}

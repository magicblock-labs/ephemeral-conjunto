use jsonrpsee::{
    core::{traits::ToRpcParams, JsonRawValue},
    types::Params,
};
use serde::{Deserialize, Serialize};
use solana_rpc_client_api::config::RpcSendTransactionConfig;

// -----------------
// RawParams
// -----------------
pub struct RawParams(pub Params<'static>);

impl ToRpcParams for RawParams {
    fn to_rpc_params(
        self,
    ) -> Result<Option<Box<JsonRawValue>>, serde_json::Error> {
        match self.0.as_str() {
            Some(s) => {
                let raw_value = JsonRawValue::from_string(s.to_string())?;
                Ok(Some(raw_value))
            }
            None => Ok(None),
        }
    }
}

// -----------------
// SendTransactionParams
// -----------------
#[derive(Debug, Deserialize, Serialize)]
pub struct SendTransactionParams(
    pub String,
    #[serde(default)] pub Option<RpcSendTransactionConfig>,
);

impl ToRpcParams for SendTransactionParams {
    fn to_rpc_params(
        self,
    ) -> Result<Option<Box<JsonRawValue>>, serde_json::Error> {
        let raw_value =
            JsonRawValue::from_string(serde_json::to_string(&self)?)?;
        Ok(Some(raw_value))
    }
}

use conjunto_addresses::cluster::RpcCluster;
use conjunto_core::{
    AccountProvider, RequestEndpoint, SignatureStatusProvider,
};
use conjunto_guidepoint::GuideStrategyResolver;
use conjunto_providers::{
    rpc_account_provider::RpcAccountProvider,
    rpc_provider_config::RpcProviderConfig,
    rpc_signature_status_provider::RpcSignatureStatusProvider,
};
use log::*;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

use crate::{
    errors::DirectorPubsubResult,
    guide_strategy::guide_strategy_from_pubsub_msg, BackendWebSocket,
};

pub struct DirectorPubsubConfig {
    pub chain_cluster: RpcCluster,
    pub ephem_rpc_provider_config: RpcProviderConfig,
}

impl DirectorPubsubConfig {
    pub fn devnet() -> Self {
        Self {
            chain_cluster: RpcCluster::Devnet,
            ephem_rpc_provider_config: RpcProviderConfig::magicblock_devnet(),
        }
    }
}

pub struct DirectorPubsub<T: AccountProvider, U: SignatureStatusProvider> {
    config: DirectorPubsubConfig,
    guide_strategy_resolver: GuideStrategyResolver<T, U>,
}

impl<T: AccountProvider, U: SignatureStatusProvider> DirectorPubsub<T, U> {
    pub fn new(
        config: DirectorPubsubConfig,
    ) -> DirectorPubsub<RpcAccountProvider, RpcSignatureStatusProvider> {
        let ephemeral_account_provider: RpcAccountProvider =
            RpcAccountProvider::new(config.ephem_rpc_provider_config.clone());
        let ephemeral_signature_status_provider =
            RpcSignatureStatusProvider::new(
                config.ephem_rpc_provider_config.clone(),
            );
        DirectorPubsub::with_providers(
            config,
            ephemeral_account_provider,
            ephemeral_signature_status_provider,
        )
    }

    pub fn with_providers(
        config: DirectorPubsubConfig,
        ephemeral_account_provider: T,
        ephemeral_signature_status_provider: U,
    ) -> Self {
        let guide_strategy_resolver = GuideStrategyResolver::new(
            ephemeral_account_provider,
            ephemeral_signature_status_provider,
        );
        Self {
            config,
            guide_strategy_resolver,
        }
    }

    pub(super) async fn guide_msg(
        &self,
        msg: &Message,
    ) -> Option<RequestEndpoint> {
        use Message::*;
        let msg = match msg {
            Text(txt) => txt,
            // When the client is trying to close the connection we attempt to do this
            // for both endpoints to get the proper response from at least one
            Close(code) => {
                debug!("Close client: {:?}", code);
                return Some(RequestEndpoint::Both);
            }
            // We don't know which chain the ping/pong msg is responding to
            // at this point, so we send to both
            Ping(_) => return Some(RequestEndpoint::Both),
            Pong(_) => return Some(RequestEndpoint::Both),

            // If in doubt just pass on to chain
            Binary(_) => return Some(RequestEndpoint::Chain),
            Frame(_) => return Some(RequestEndpoint::Chain),
        };
        let strategy = guide_strategy_from_pubsub_msg(msg.as_str());
        let endpoint = self.guide_strategy_resolver.resolve(&strategy).await;
        trace!("Message '{}", msg);
        debug!("Guiding message to: {:?}", endpoint);
        Some(endpoint)
    }

    pub async fn try_chain_client(
        &self,
    ) -> DirectorPubsubResult<BackendWebSocket> {
        let url = self.config.chain_cluster.ws_url();
        let (socket, _) = connect_async(Url::parse(url)?).await?;
        Ok(socket)
    }

    pub async fn try_ephemeral_client(
        &self,
    ) -> DirectorPubsubResult<BackendWebSocket> {
        let url = self.config.ephem_rpc_provider_config.cluster().ws_url();
        let (socket, _) = connect_async(Url::parse(url)?).await?;
        Ok(socket)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use conjunto_test_tools::{
        account_provider_stub::AccountProviderStub,
        signature_status_provider_stub::SignatureStatusProviderStub,
    };
    use serde_json::Value;
    use solana_sdk::signature::Signature;

    use super::*;

    async fn guide_and_assert(
        director: &DirectorPubsub<
            AccountProviderStub,
            SignatureStatusProviderStub,
        >,
        msg_val: Value,
        expected: &RequestEndpoint,
    ) {
        let msg = Message::Text(msg_val.to_string());
        let actual = director.guide_msg(&msg).await.unwrap();
        assert_eq!(&actual, expected);
    }

    // -----------------
    // subscribeSignature
    // -----------------
    fn subscribe_signature() -> Value {
        serde_json::json! {{
            "method": "signatureSubscribe",
            "params": [
                "2EBVM6cB8vAAD93Ktr6Vd8p67XPbQzCJX47MpReuiCXJAtcjaxpvWpcg9Ege1Nr5Tk3a2GFrByT7WPBjdsTycY9b", {
                "commitment": "finalized",
                "enableReceivedNotification": false
            }]
        }}
    }

    fn signature() -> Signature {
        Signature::from_str("2EBVM6cB8vAAD93Ktr6Vd8p67XPbQzCJX47MpReuiCXJAtcjaxpvWpcg9Ege1Nr5Tk3a2GFrByT7WPBjdsTycY9b").unwrap()
    }

    #[tokio::test]
    async fn test_guide_subscribe_signature_found_in_ephemeral() {
        let mut signature_status_provider =
            SignatureStatusProviderStub::default();
        signature_status_provider.add_ok(signature());

        let director = DirectorPubsub::with_providers(
            DirectorPubsubConfig::devnet(),
            AccountProviderStub::default(),
            signature_status_provider,
        );
        guide_and_assert(
            &director,
            subscribe_signature(),
            &RequestEndpoint::Ephemeral,
        )
        .await;
    }

    #[tokio::test]
    async fn test_guide_subscribe_signature_not_found_in_ephemeral() {
        let signature_status_provider = SignatureStatusProviderStub::default();

        let director = DirectorPubsub::with_providers(
            DirectorPubsubConfig::devnet(),
            AccountProviderStub::default(),
            signature_status_provider,
        );
        guide_and_assert(
            &director,
            subscribe_signature(),
            &RequestEndpoint::Both,
        )
        .await;
    }

    #[tokio::test]
    async fn test_guide_subscribe_signature_invalid_signature() {
        let signature_status_provider = SignatureStatusProviderStub::default();

        let director = DirectorPubsub::with_providers(
            DirectorPubsubConfig::devnet(),
            AccountProviderStub::default(),
            signature_status_provider,
        );
        let subscribe = serde_json::json! {{
            "method": "signatureSubscribe",
            "params": [ "<not a valid signature>"]
        }};
        guide_and_assert(&director, subscribe, &RequestEndpoint::Chain).await;
    }

    // TODO(thlorenz): Add more tests for other pubsub messages
}

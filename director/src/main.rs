use conjunto_director_pubsub::{
    director::DirectorPubsubConfig, start_pubsub_server,
};
use conjunto_director_rpc::{rpc::DirectorConfig, start_rpc_server};
use conjunto_providers::{
    rpc_account_provider::RpcAccountProvider,
    rpc_signature_status_provider::RpcSignatureStatusProvider,
};
use log::*;

#[tokio::main]
async fn main() {
    env_logger::init();

    let (rpc_addr, rpc_handle) =
        start_rpc_server(DirectorConfig::devnet(), None)
            .await
            .unwrap();

    let (pubsub_addr, pubsub_handle) =
        start_pubsub_server::<RpcAccountProvider, RpcSignatureStatusProvider>(
            DirectorPubsubConfig::devnet(),
            None,
        )
        .await
        .unwrap();
    info!("RPC Server running on: {}", rpc_addr);
    info!("Pubsub Server running on: {}", pubsub_addr);

    let (_, res) = tokio::join!(rpc_handle.stopped(), pubsub_handle);
    if let Err(err) = res {
        error!("Error: {:?}", err);
    }
}

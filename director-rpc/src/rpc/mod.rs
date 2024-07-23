use conjunto_addresses::cluster::RpcCluster;
use conjunto_providers::rpc_provider_config::RpcProviderConfig;
use conjunto_transwise::Transwise;
use jsonrpsee::{
    http_client::{HttpClient, HttpClientBuilder},
    RpcModule,
};

use self::{
    guide::register_guide_methods, passthrough::register_passthrough_methods,
};
use crate::errors::DirectorRpcResult;

pub mod guide;
mod params;
pub mod passthrough;

pub struct DirectorConfig {
    pub ephem_rpc_provider_config: RpcProviderConfig,
    pub chain_cluster: RpcCluster,
}

impl DirectorConfig {
    pub fn devnet() -> Self {
        Self {
            chain_cluster: RpcCluster::Devnet,
            ephem_rpc_provider_config: RpcProviderConfig::magicblock_devnet(),
        }
    }
}

pub struct DirectorRpc {
    pub(super) transwise: Transwise,
    pub(super) rpc_chain_client: HttpClient,
    pub(super) rpc_ephem_client: HttpClient,
}

pub fn create_rpc_module(
    config: DirectorConfig,
) -> DirectorRpcResult<RpcModule<DirectorRpc>> {
    let ephem_url = config.ephem_rpc_provider_config.url().to_string();
    let transwise = Transwise::new(config.ephem_rpc_provider_config);

    let rpc_ephem_client = HttpClientBuilder::default().build(ephem_url)?;
    let rpc_chain_client =
        HttpClientBuilder::default().build(config.chain_cluster.url())?;

    let director = DirectorRpc {
        transwise,
        rpc_ephem_client,
        rpc_chain_client,
    };

    let mut module = RpcModule::new(director);

    register_guide_methods(&mut module)?;
    register_passthrough_methods(&mut module)?;

    Ok(module)
}

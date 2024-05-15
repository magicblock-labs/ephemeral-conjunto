use conjunto_addresses::cluster::RpcCluster;
use solana_sdk::commitment_config::CommitmentLevel;

#[derive(Default, Clone)]
pub struct RpcProviderConfig {
    cluster: RpcCluster,
    commitment: Option<CommitmentLevel>,
}

impl RpcProviderConfig {
    pub fn new(
        cluster: RpcCluster,
        commitment: Option<CommitmentLevel>,
    ) -> Self {
        Self {
            cluster,
            commitment,
        }
    }

    pub fn cluster(&self) -> &RpcCluster {
        &self.cluster
    }

    pub fn url(&self) -> &str {
        self.cluster.url()
    }

    pub fn ws_url(&self) -> &str {
        self.cluster.ws_url()
    }

    pub fn commitment(&self) -> Option<CommitmentLevel> {
        self.commitment
    }
}

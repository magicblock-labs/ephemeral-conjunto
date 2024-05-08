pub const DEVNET: &str = "https://api.devnet.solana.com";
pub const MAINNET: &str = "https://api.mainnet-beta.solana.com";
pub const TESTNET: &str = "https://api.testnet.solana.com";
pub const DEVELOPMENT: &str = "http://localhost:8899";

#[derive(Default)]
pub enum RpcCluster {
    #[default]
    Devnet,
    Mainnet,
    Testnet,
    Development,
    Custom(String),
}

impl RpcCluster {
    pub fn url(&self) -> &str {
        match self {
            RpcCluster::Devnet => DEVNET,
            RpcCluster::Mainnet => MAINNET,
            RpcCluster::Testnet => TESTNET,
            RpcCluster::Development => DEVELOPMENT,
            RpcCluster::Custom(url) => url,
        }
    }
}

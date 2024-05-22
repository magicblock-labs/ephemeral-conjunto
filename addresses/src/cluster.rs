pub const DEVNET: &str = "https://api.devnet.solana.com";
pub const MAINNET: &str = "https://api.mainnet-beta.solana.com";
pub const TESTNET: &str = "https://api.testnet.solana.com";
pub const DEVELOPMENT: &str = "http://localhost:8899";

pub const WS_DEVNET: &str = "wss://api.devnet.solana.com/";
pub const WS_MAINNET: &str = "wss://api.mainnet-beta.solana.com/";
pub const WS_TESTNET: &str = "wss://api.testnet.solana.com/";
pub const WS_DEVELOPMENT: &str = "ws://localhost:8900";

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum RpcCluster {
    #[default]
    Devnet,
    Mainnet,
    Testnet,
    Development,
    Custom(String, String),
}

impl RpcCluster {
    pub fn url(&self) -> &str {
        match self {
            RpcCluster::Devnet => DEVNET,
            RpcCluster::Mainnet => MAINNET,
            RpcCluster::Testnet => TESTNET,
            RpcCluster::Development => DEVELOPMENT,
            RpcCluster::Custom(url, _) => url,
        }
    }

    pub fn ws_url(&self) -> &str {
        match self {
            RpcCluster::Devnet => WS_DEVNET,
            RpcCluster::Mainnet => WS_MAINNET,
            RpcCluster::Testnet => WS_TESTNET,
            RpcCluster::Development => WS_DEVELOPMENT,
            RpcCluster::Custom(_, ws_url) => ws_url,
        }
    }
}

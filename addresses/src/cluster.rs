pub const MAINNET: &str = "https://api.mainnet-beta.solana.com";
pub const TESTNET: &str = "https://api.testnet.solana.com";
pub const DEVNET: &str = "https://api.devnet.solana.com";
pub const DEVELOPMENT: &str = "http://localhost:8899";

pub const WS_MAINNET: &str = "wss://api.mainnet-beta.solana.com/";
pub const WS_TESTNET: &str = "wss://api.testnet.solana.com/";
pub const WS_DEVNET: &str = "wss://api.devnet.solana.com/";
pub const WS_DEVELOPMENT: &str = "ws://localhost:8900";

pub const MAGICBLOCK_DEVNET: &str = "https://devnet.magicblock.app";
pub const MAGICBLOCK_WS_DEVNET: &str = "wss://devnet.magicblock.app:8900";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpcCluster {
    Mainnet,
    Testnet,
    Devnet,
    Development,
    Custom(String, String),
}

impl RpcCluster {
    pub fn url(&self) -> &str {
        match self {
            RpcCluster::Mainnet => MAINNET,
            RpcCluster::Testnet => TESTNET,
            RpcCluster::Devnet => DEVNET,
            RpcCluster::Development => DEVELOPMENT,
            RpcCluster::Custom(url, _) => url,
        }
    }

    pub fn ws_url(&self) -> &str {
        match self {
            RpcCluster::Mainnet => WS_MAINNET,
            RpcCluster::Testnet => WS_TESTNET,
            RpcCluster::Devnet => WS_DEVNET,
            RpcCluster::Development => WS_DEVELOPMENT,
            RpcCluster::Custom(_, ws_url) => ws_url,
        }
    }

    pub fn magicblock_devnet() -> Self {
        Self::Custom(
            MAGICBLOCK_DEVNET.to_string(),
            MAGICBLOCK_WS_DEVNET.to_string(),
        )
    }
}

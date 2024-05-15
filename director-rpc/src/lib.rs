mod decoders;
pub mod errors;
pub mod rpc;
mod utils;

use std::net::SocketAddr;

use errors::DirectorRpcResult;
use jsonrpsee::server::{Server, ServerHandle};
use rpc::{create_rpc_module, DirectorConfig};
use tower_http::cors::{Any, CorsLayer};

pub const DEFAULT_DIRECTOR_RPC_URL: &str = "127.0.0.1:9899";

pub async fn start_rpc_server(
    config: DirectorConfig,
    url: Option<&str>,
) -> DirectorRpcResult<(String, ServerHandle)> {
    let url = url.unwrap_or(DEFAULT_DIRECTOR_RPC_URL);

    // NOTE: we tried to add proper middleware here, but run into some trait
    // implementation issues at server.start()
    // Stopped at this point because we might let jsonrpsee die in fire and
    // implement a lower level approach similarly how we did for pubsub which
    // would also allow us to control what things get parsed and how.
    // At that point we can try to add CORS support again
    let _cors_middleware = {
        let cors = CorsLayer::new()
            .allow_methods([hyper::Method::POST])
            .allow_origin(Any)
            .allow_headers([hyper::header::CONTENT_TYPE]);
        tower::ServiceBuilder::new().layer(cors)
    };

    let server = Server::builder()
        // .set_http_middleware(cors_middleware)
        .http_only()
        .build(url.parse::<SocketAddr>().unwrap())
        .await?;

    let rpc_module = create_rpc_module(config)?;
    let handle = server.start(rpc_module);
    Ok((url.to_string(), handle))
}

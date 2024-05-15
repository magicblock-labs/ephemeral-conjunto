use conjunto_core::{AccountProvider, SignatureStatusProvider};
use futures_util::stream::SplitSink;
use log::*;
use std::sync::Arc;

use director::{DirectorPubsub, DirectorPubsubConfig};
use errors::DirectorPubsubResult;
use tokio::{
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};
use tokio_tungstenite::{
    tungstenite::Message, MaybeTlsStream, WebSocketStream,
};

mod accept_connection;
mod director;
pub mod errors;
mod guide_strategy;
mod messages;

pub type BackendWebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type BackendWebSocketWriter =
    SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

pub const DEFAULT_DIRECTOR_PUBSUB_URL: &str = "127.0.0.1:9900";

pub async fn start_pubsub_server<
    T: AccountProvider,
    U: SignatureStatusProvider,
>(
    config: DirectorPubsubConfig,
    url: Option<&str>,
) -> DirectorPubsubResult<(String, JoinHandle<()>)> {
    let url = url.unwrap_or(DEFAULT_DIRECTOR_PUBSUB_URL);
    let listener = TcpListener::bind(&url).await?;
    let director = Arc::new(DirectorPubsub::<T, U>::new(config));
    let pubsub_handle = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let chain_client = match director.try_chain_client().await {
                Err(err) => {
                    error!("Failed to connect to chain client: {}", err);
                    continue;
                }
                Ok(client) => client,
            };
            let ephem_client = match director.try_ephemeral_client().await {
                Err(err) => {
                    error!("Failed to connect to ephemeral client: {}", err);
                    continue;
                }
                Ok(client) => client,
            };
            tokio::spawn(accept_connection::accept_connection(
                director.clone(),
                chain_client,
                ephem_client,
                stream,
            ));
        }
    });

    Ok((url.to_string(), pubsub_handle))
}

use std::sync::Arc;

use crate::{
    director::DirectorPubsub, errors::DirectorPubsubResult, BackendWebSocket,
    BackendWebSocketWriter,
};
use conjunto_core::{
    AccountProvider, RequestEndpoint, SignatureStatusProvider,
};
use futures_util::{SinkExt, StreamExt};
use log::*;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;

pub(crate) async fn accept_connection<
    T: AccountProvider,
    U: SignatureStatusProvider,
>(
    director: Arc<DirectorPubsub<T, U>>,
    chain_socket: BackendWebSocket,
    ephem_socket: BackendWebSocket,
    incoming_stream: TcpStream,
) -> DirectorPubsubResult<()> {
    let addr = incoming_stream.peer_addr()?;
    debug!("Peer address: {}", addr);

    let client_stream =
        tokio_tungstenite::accept_async(incoming_stream).await?;

    let (mut write_client, mut read_client) = client_stream.split();
    let (mut write_chain, mut read_chain) = chain_socket.split();
    let (mut write_ephem, mut read_ephem) = ephem_socket.split();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                // We pipe both chain and ephemeral messages to the client
                next = read_chain.next() => {
                    match next {
                        Some(Ok(msg)) => {
                            trace!("Chain message: {:?}", msg);
                            let res = handle_downstream_msg(&mut write_chain, &msg).await;
                            if res.fwd_to_client {
                                write_client.send(msg).await.unwrap();
                            }
                            if res.done {
                                break;
                            }
                        }
                        Some(Err(msg)) => {
                            // We get a Protocol(ResetWithoutClosingHandshake) right before
                            // the chain stream gets interrupted for subscriptions
                            trace!("Error reading chain message: {:?}", msg);
                        }
                        None => {
                            // If either downstream disconnects we need to make the client
                            // aware and thus disconnect ourselves as well
                            break;
                        }
                    }
                }
                next = read_ephem.next() => {
                    match next {
                        Some(Ok(msg)) => {
                            trace!("Ephem message: {:?}", msg);
                            let res = handle_downstream_msg(&mut write_ephem, &msg).await;
                            if res.fwd_to_client {
                                write_client.send(msg).await.unwrap();
                            }
                            if res.done {
                                break;
                            }
                        }
                        Some(Err(msg)) => {
                            trace!("Error reading ephem message: {:?}", msg);
                        }
                        None => {
                            break;
                        }
                    }
                }
                // For client messages we decide by message content if to send it
                // to chain or ephem socket
                next = read_client.next() => {
                    match next {
                        Some(Ok(msg)) => {
                            trace!("Client message: {:?}", msg);
                            use RequestEndpoint::*;
                            match director.guide_msg(&msg).await {
                                Some(Chain) => {
                                    trace!("Sending message to chain: {:?}", msg);
                                    write_chain.send(msg).await.unwrap()
                                },
                                Some(Ephemeral) => {
                                    trace!("Sending message to ephemeral: {:?}", msg);
                                    write_ephem.send(msg).await.unwrap();
                                }
                                Some(Both) => {
                                    trace!("Sending message to chain and ephemeral: {:?}", msg);
                                    write_chain.send(msg.clone()).await.unwrap();
                                    write_ephem.send(msg).await.unwrap();
                                }
                                // If client sends a "close" message we return None as endpoint
                                None => break
                            }
                        }
                        Some(Err(err)) => {
                            error!("Error reading client message: {:?}", err);
                            break;
                        }
                        None => {
                            debug!("Client stream ended");
                            break;
                        }
                    }
                },
            };
        }
    });
    Ok(())
}

struct HandleDownstreamMsgResult {
    done: bool,
    fwd_to_client: bool,
}
impl HandleDownstreamMsgResult {
    fn not_done_fwd() -> Self {
        Self {
            done: false,
            fwd_to_client: true,
        }
    }
    fn not_done_no_fwd() -> Self {
        Self {
            done: false,
            fwd_to_client: false,
        }
    }
    fn done_fwd() -> Self {
        Self {
            done: true,
            fwd_to_client: true,
        }
    }
}
async fn handle_downstream_msg(
    ws: &mut BackendWebSocketWriter,
    msg: &Message,
) -> HandleDownstreamMsgResult {
    match msg {
        Message::Text(_) => HandleDownstreamMsgResult::not_done_fwd(),
        Message::Binary(_data) => HandleDownstreamMsgResult::not_done_fwd(),
        Message::Ping(data) => {
            // Need to respond in order to keep the socket connection open, otherwise
            // the downstream (mainnet/devnet) may close the connection
            // See https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API/Writing_WebSocket_servers#pings_and_pongs_the_heartbeat_of_websockets
            if let Err(err) = ws.send(Message::Pong(data.clone())).await {
                trace!("Failed to send pong: {:?}", err);
            }
            HandleDownstreamMsgResult::not_done_no_fwd()
        }
        Message::Pong(_data) => HandleDownstreamMsgResult::not_done_fwd(),
        Message::Close(_frame) => HandleDownstreamMsgResult::done_fwd(),
        Message::Frame(_frame) => HandleDownstreamMsgResult::not_done_fwd(),
    }
}

//! # Betfair Stream Server Mock
//!
//! This crate provides a mock implementation of the Betfair Stream API,
//! allowing for testing and development of applications that interact with
//! the Betfair streaming service. It handles TLS connections, client state
//! management, and message encoding/decoding.
//!
//! ## Features
//! - Integration testing support
//! - TLS support for secure connections
//! - Client state management
//! - Message handling for authentication and subscriptions

mod tls;

use core::net::SocketAddr;
use std::sync::Arc;

use betfair_stream_types::request::authentication_message::AuthenticationMessage;
use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::response::connection_message::ConnectionMessage;
use betfair_stream_types::response::status_message::StatusMessage;
use betfair_stream_types::response::ResponseMessage;
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;
use tokio_util::bytes;
use tokio_util::codec::{Decoder, Encoder, Framed};
use url::Url;

/// Represents the backend for the Stream API, handling connections and configurations.
pub struct StreamAPIBackend {
    /// The address the listener is bound to.
    pub listener_addr: SocketAddr,
    /// The TCP listener for incoming connections.
    pub listener: TcpListener,
    /// The server configuration for TLS.
    pub server_config: Arc<rustls::ServerConfig>,
    /// The certificate used for TLS connections.
    pub cert: String,
    /// The URL for the Stream API.
    pub url: Url,
}

impl StreamAPIBackend {
    /// Creates a new instance of `StreamAPIBackend`.
    /// This function is only available when the "integration-test" feature is enabled.
    #[cfg(feature = "integration-test")]
    pub async fn new() -> Self {
        use betfair_stream_api::CERTIFICATE;
        let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
        let listener_addr = listener.local_addr().unwrap();
        let (cert, key) = tls::generate_cert().unwrap();
        let server_config = tls::rustls_config(cert.as_str(), key.as_str());

        let url = Url::parse(&format!("http://{listener_addr}")).unwrap();

        let _ = CERTIFICATE.set(cert.clone());

        Self {
            listener_addr,
            listener,
            server_config,
            cert,
            url,
        }
    }

    /// Accepts the next incoming connection and returns a `ClientStateW`.
    pub async fn process_next(&self) -> ClientStateW {
        let acceptor = TlsAcceptor::from(Arc::clone(&self.server_config));
        let (socket, _tt) = self.listener.accept().await.unwrap();
        let tls_stream = acceptor.accept(socket).await.unwrap();

        tracing::info!("Accepted connection");

        let client_state = Arc::new(tokio::sync::Mutex::new(ClientState::Init(
            ConnState::Connected,
        )));

        ClientStateW::new(tls_stream, client_state)
    }
}

/// Represents the state of a client connection.
pub struct ClientStateW {
    /// The TLS stream for the client connection.
    socket: TlsStream<TcpStream>,
    /// The current state of the client, wrapped in a mutex for safe concurrent access.
    pub state: Arc<tokio::sync::Mutex<ClientState>>,
}

impl ClientStateW {
    /// Creates a new instance of `ClientStateW`.
    pub const fn new(
        socket: TlsStream<TcpStream>,
        state: Arc<tokio::sync::Mutex<ClientState>>,
    ) -> Self {
        Self { socket, state }
    }

    /// Processes incoming messages from the client and manages the client state.
    pub async fn process(self) {
        let mut socket = Framed::new(self.socket, StreamAPIServerCodec);
        loop {
            tokio::time::sleep(core::time::Duration::from_millis(100)).await;
            let client_state = self.state.lock().await.clone();

            let new_client_state = match client_state {
                ClientState::Init(ConnState::Connected) => {
                    socket
                        .feed(ResponseMessage::Connection(ConnectionMessage {
                            id: Some(1),
                            connection_id: Some("conn_id_fake123".to_owned()),
                        }))
                        .await
                        .unwrap();
                    ClientState::Init(ConnState::WaitingForAuthInfo)
                }
                ClientState::Init(ConnState::WaitingForAuthInfo) => {
                    tracing::warn!("WAITING FOR MESSAGE");
                    let msg = socket
                        .next()
                        .await
                        .transpose()
                        .expect("client stream closed!");
                    tracing::info!("ConnState::WaitingForAuthInfo: Received message {msg:?}");
                    let Some(msg) = msg else { continue };
                    let RequestMessage::Authentication(AuthenticationMessage {
                        id,
                        app_key: _,
                        session: _,
                    }) = msg
                    else {
                        panic!("Unexpected message");
                    };

                    socket
                        .feed(ResponseMessage::Status(StatusMessage {
                            id,
                            connection_closed: None,
                            connection_id: Some("conn_id_fake123".to_owned()),
                            connections_available: Some(42),
                            error_code: None,
                            error_message: None,
                            status_code: Some(
                                betfair_stream_types::response::status_message::StatusCode::Success,
                            ),
                        }))
                        .await
                        .unwrap();
                    ClientState::LoggedIn(SubSate {
                        heartbeat_counter: 0,
                    })
                }
                ClientState::LoggedIn(mut state) => {
                    tracing::info!("Waiting for message");
                    let msg = socket.next().await.transpose().unwrap();
                    tracing::info!("Received message {msg:?}");
                    let Some(msg) = msg else { continue };
                    let sub_state = match msg {
                        RequestMessage::Authentication(_) => todo!(),
                        RequestMessage::Heartbeat(_hb) => {
                            state.heartbeat_counter += 1;
                            socket
                                .feed(ResponseMessage::Status(StatusMessage {
                                    id: Some(1),
                                    connection_closed: None,
                                    connection_id: Some("conn_id_fake123".to_owned()),
                                    connections_available: None,
                                    error_code: None,
                                    error_message: None,
                                    status_code: None,
                                }))
                                .await
                                .unwrap();
                            state
                        }
                        RequestMessage::MarketSubscription(_) => todo!(),
                        RequestMessage::OrderSubscription(_) => todo!(),
                    };
                    ClientState::LoggedIn(sub_state)
                }
            };

            socket.flush().await.unwrap();
            let mut client_state = self.state.lock().await;
            *client_state = new_client_state;
            drop(client_state);
        }
    }
}

/// Represents the connection states for a client.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum ConnState {
    /// The client is connected.
    Connected,
    /// The client is waiting for authentication information.
    WaitingForAuthInfo,
}

/// Represents the state of a logged-in client, including a heartbeat counter.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct SubSate {
    /// The number of heartbeats sent by the client.
    pub heartbeat_counter: u64,
}

/// Represents the various states a client can be in.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum ClientState {
    /// The client is initializing.
    Init(ConnState),
    /// The client is logged in.
    LoggedIn(SubSate),
}

/// Codec for encoding and decoding messages for the Stream API.
pub struct StreamAPIServerCodec;

impl Decoder for StreamAPIServerCodec {
    type Item = RequestMessage;
    type Error = eyre::Error;

    /// Decodes a message from the provided buffer.
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Check if there is a newline character in the buffer
        if let Some(newline_index) = src.iter().position(|&b| b == b'\n') {
            // Extract the message up to the newline character
            let line = src.split_to(newline_index + 1);

            // Deserialize the JSON data
            let data = serde_json::from_slice::<Self::Item>(&line[..line.len() - 1])?;

            Ok(Some(data))
        } else {
            // Not enough data to read a complete line
            Ok(None)
        }
    }
}

impl Encoder<ResponseMessage> for StreamAPIServerCodec {
    type Error = eyre::Error;

    /// Encodes a response message into the provided buffer.
    fn encode(
        &mut self,
        item: ResponseMessage,
        dst: &mut bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        // Serialize the item to a JSON string
        let json = serde_json::to_string(&item)?;
        // Write the JSON string to the buffer, followed by a newline
        dst.extend_from_slice(json.as_bytes());
        dst.extend_from_slice(b"\r\n");
        Ok(())
    }
}

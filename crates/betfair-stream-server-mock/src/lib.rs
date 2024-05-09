// #![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
// #![deny(unused_must_use, rust_2018_idioms)]
// #![doc(test(
//     no_crate_inject,
//     attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
// ))]

mod tls;

use std::net::SocketAddr;
use std::sync::Arc;

use betfair_stream_api::CERTIFICATE;
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

pub struct StreamAPIBackend {
    pub listener_addr: SocketAddr,
    pub listener: TcpListener,
    pub server_config: Arc<rustls::ServerConfig>,
    pub cert: String,
    pub url: Url,
}

impl StreamAPIBackend {
    pub async fn new() -> Self {
        let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
        let listener_addr = listener.local_addr().unwrap();
        let (cert, key) = tls::generate_cert().unwrap();
        let server_config = tls::rustls_config(cert.as_str(), key.as_str());

        let url = Url::parse(&format!("http://{}", listener_addr)).unwrap();

        CERTIFICATE.set(cert.clone()).unwrap();

        Self {
            listener_addr,
            cert,
            server_config,
            url,
            listener,
        }
    }

    pub async fn process_next(&self) -> ClientStateW {
        let acceptor = TlsAcceptor::from(Arc::clone(&self.server_config));
        let (socket, _tt) = self.listener.accept().await.unwrap();
        let tls_stream = acceptor.accept(socket).await.unwrap();

        tracing::info!("Accepted connection");

        let client_state = Arc::new(tokio::sync::Mutex::new(ClientState::Init(
            ConnState::Connected,
        )));

        ClientStateW::new(tls_stream, client_state.clone())
    }
}

pub struct ClientStateW {
    socket: TlsStream<TcpStream>,
    pub state: Arc<tokio::sync::Mutex<ClientState>>,
}

impl ClientStateW {
    pub fn new(socket: TlsStream<TcpStream>, state: Arc<tokio::sync::Mutex<ClientState>>) -> Self {
        Self { socket, state }
    }

    pub async fn process(self) {
        let mut socket = Framed::new(self.socket, StreamAPIServerCodec);
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let client_state = self.state.lock().await.clone();

            let new_client_state = match client_state {
                ClientState::Init(ConnState::Connected) => {
                    socket
                        .feed(ResponseMessage::Connection(ConnectionMessage {
                            id: Some(1),
                            connection_id: Some("conn_id_fake123".to_string()),
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
                            connection_id: Some("conn_id_fake123".to_string()),
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
                                    connection_id: Some("conn_id_fake123".to_string()),
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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ConnState {
    Connected,
    WaitingForAuthInfo,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct SubSate {
    pub heartbeat_counter: u64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ClientState {
    Init(ConnState),
    LoggedIn(SubSate),
}

pub struct StreamAPIServerCodec;

impl Decoder for StreamAPIServerCodec {
    type Item = RequestMessage;
    type Error = eyre::Error;

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

use std::net::SocketAddr;
use std::sync::Arc;

use betfair_stream_types::request::authentication_message::AuthenticationMessage;
use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::response::connection_message::ConnectionMessage;
use betfair_stream_types::response::status_message::StatusMessage;
use betfair_stream_types::response::ResponseMessage;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use url::Url;

pub struct StreamAPIBackend {
    pub listener_addr: SocketAddr,
    pub listener: TcpListener,
    pub url: Url,
}

impl StreamAPIBackend {
    pub async fn new() -> Self {
        let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
        let listener_addr = listener.local_addr().unwrap();

        tokio::spawn(async move {});

        let url = Url::parse(&format!("http://{}", listener_addr)).unwrap();
        Self {
            listener_addr,
            url,
            listener,
        }
    }

    pub async fn process_next(&self) -> ClientStateW {
        let (socket, _) = self.listener.accept().await.unwrap();
        tracing::info!("Accepted connection");

        let client_state = Arc::new(tokio::sync::RwLock::new(ClientState::Init(
            ConnState::Connected,
        )));

        ClientStateW::new(socket, client_state.clone())
    }
}

pub struct ClientStateW {
    socket: TcpStream,
    pub state: Arc<tokio::sync::RwLock<ClientState>>,
}

impl ClientStateW {
    pub fn new(socket: TcpStream, state: Arc<tokio::sync::RwLock<ClientState>>) -> Self {
        Self { socket, state }
    }

    pub async fn process(self) {
        let mut reader = BufReader::new(self.socket);
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let mut client_state = self.state.write().await;
            match &mut *client_state {
                ClientState::Init(ConnState::Connected) => {
                    send(
                        &mut reader,
                        &ResponseMessage::Connection(ConnectionMessage {
                            id: Some(1),
                            connection_id: Some("123".to_string()),
                        }),
                    )
                    .await;
                    *client_state = ClientState::Init(ConnState::WaitingForAuthInfo);
                }
                ClientState::Init(ConnState::WaitingForAuthInfo) => {
                    let msg = recv(&mut reader).await;
                    let Some(msg) = msg else {
                        continue
                    };
                    let RequestMessage::Authentication(AuthenticationMessage {
                        id,
                        app_key: _,
                        session: _,
                    }) = msg
                    else {
                        panic!("Unexpected message");
                    };

                    send(
                        &mut reader,
                        &ResponseMessage::StatusMessage(StatusMessage {
                            id,
                            connection_closed: None,
                            connection_id: Some("123".to_string()),
                            connections_available: None,
                            error_code: None,
                            error_message: None,
                            status_code: None,
                        }),
                    )
                    .await;
                    *client_state = ClientState::LoggedIn(SubSate {
                        keep_alive_counter: 0,
                    });
                }
                ClientState::LoggedIn(ref mut state) => {
                    tracing::info!("Waiting for message");
                    let msg = recv(&mut reader).await;
                    tracing::info!("Received message {msg:?}");
                    let Some(msg) = msg else {
                        continue
                    };
                    match msg {
                        RequestMessage::Authentication(_) => todo!(),
                        RequestMessage::Heartbeat(_hb) => {
                            state.keep_alive_counter += 1;
                            send(
                                &mut reader,
                                &ResponseMessage::StatusMessage(StatusMessage {
                                    id: Some(1),
                                    connection_closed: None,
                                    connection_id: Some("123".to_string()),
                                    connections_available: None,
                                    error_code: None,
                                    error_message: None,
                                    status_code: None,
                                }),
                            )
                            .await;
                        }
                        RequestMessage::MarketSubscription(_) => todo!(),
                        RequestMessage::OrderSubscription(_) => todo!(),
                    }
                }
            }

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
    pub keep_alive_counter: u64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ClientState {
    Init(ConnState),
    LoggedIn(SubSate),
}

async fn send(reader: &mut BufReader<TcpStream>, message: &ResponseMessage) {
    let message = serde_json::to_string(message).unwrap() + "\r\n";
    reader.write_all(message.as_bytes()).await.unwrap();
}
async fn recv(reader: &mut BufReader<TcpStream>) -> Option<RequestMessage> {
    let mut buf = String::new();
    reader.read_line(&mut buf).await.unwrap();
    if buf.is_empty() {
        return None
    }

    tracing::info!(buf = ?buf, "Received message");
    let buf = &buf[..buf.len() - 2];
    Some(serde_json::from_str::<RequestMessage>(buf).unwrap())
}

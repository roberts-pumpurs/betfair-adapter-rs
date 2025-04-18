//! Betfair stream client
extern crate alloc;
pub mod cache;
use backon::{BackoffBuilder, ExponentialBuilder};
use betfair_adapter::{ApplicationKey, Authenticated, BetfairRpcClient, SessionToken};
pub use betfair_stream_types as types;
use betfair_stream_types::{
    request::{RequestMessage, authentication_message},
    response::{
        ResponseMessage,
        connection_message::ConnectionMessage,
        status_message::{ErrorCode, StatusMessage},
    },
};
use cache::{
    primitives::{MarketBookCache, OrderBookCache},
    tracker::StreamState,
};
use eyre::{Context, OptionExt};
use futures::{
    SinkExt, StreamExt,
    future::{self, Select, select},
};
use std::{error::Error, sync::Arc};
use std::{pin::pin, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
    time::{interval, sleep},
};
use tokio_util::{
    bytes,
    codec::{Decoder, Encoder, Framed},
};

/// A Betfair Stream API client that handles connection, handshake, incoming/outgoing messages,
/// heartbeat and automatic reconnects.
pub struct BetfairStreamConnection<T: MessageProcessor> {
    /// betfair cient
    pub client: BetfairRpcClient<Authenticated>,
    /// Heartbeat interval (used only if heartbeat_enabled is true)
    pub heartbeat_interval: Option<Duration>,
    pub processor: T,
}

pub struct BetfairStreamClient<T: MessageProcessor> {
    pub send_to_stream: Sender<RequestMessage>,
    pub sink: Receiver<T::Output>,
}

pub struct Cache {
    state: StreamState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CachedMessage {
    Connection(ConnectionMessage),
    MarketChange(Vec<MarketBookCache>),
    OrderChange(Vec<OrderBookCache>),
    Status(StatusMessage),
}

impl MessageProcessor for Cache {
    type Output = CachedMessage;

    fn process_message(&mut self, message: ResponseMessage) -> Self::Output {
        match message {
            ResponseMessage::Connection(connection_message) => {
                CachedMessage::Connection(connection_message)
            }
            ResponseMessage::MarketChange(market_change_message) => {
                let data = self
                    .state
                    .market_change_update(market_change_message)
                    .into_iter()
                    .cloned()
                    .collect();

                CachedMessage::MarketChange(data)
            }
            ResponseMessage::OrderChange(order_change_message) => {
                let data = self
                    .state
                    .order_change_update(order_change_message)
                    .into_iter()
                    .cloned()
                    .collect();

                CachedMessage::OrderChange(data)
            }
            ResponseMessage::Status(status_message) => CachedMessage::Status(status_message),
        }
    }
}
pub struct Forwarder;
impl MessageProcessor for Forwarder {
    type Output = ResponseMessage;

    fn process_message(&mut self, message: ResponseMessage) -> Self::Output {
        message
    }
}
pub trait MessageProcessor: Send + Sync + 'static {
    type Output: Send + Clone + Sync + 'static;
    fn process_message(&mut self, message: ResponseMessage) -> Self::Output;
}

impl<T: MessageProcessor> BetfairStreamConnection<T> {
    pub fn new(client: BetfairRpcClient<Authenticated>) -> BetfairStreamConnection<Cache> {
        BetfairStreamConnection {
            client,
            heartbeat_interval: None,
            processor: Cache {
                state: StreamState::new(),
            },
        }
    }

    pub fn new_without_processing(
        client: BetfairRpcClient<Authenticated>,
    ) -> BetfairStreamConnection<Forwarder> {
        BetfairStreamConnection {
            client,
            heartbeat_interval: None,
            processor: Forwarder,
        }
    }

    pub async fn start(self) -> (BetfairStreamClient<T>, JoinHandle<eyre::Result<()>>) {
        let (to_stream_tx, to_stream_rx) = mpsc::channel(100);
        let (from_stream_tx, from_stream_rx) = mpsc::channel(100);

        let task = tokio::task::spawn(self.run(from_stream_tx, to_stream_rx));

        (
            BetfairStreamClient {
                send_to_stream: to_stream_tx,
                sink: from_stream_rx,
            },
            task,
        )
    }

    /// Start the client. This function will continuously try to connect to Betfair,
    /// perform the handshake, and then launch I/O tasks for processing messages.
    ///
    /// In case of a lost connection, it will reconnect using exponential backoff.
    pub async fn run(
        mut self,
        mut from_stream_tx: Sender<T::Output>,
        mut to_stream_rx: Receiver<RequestMessage>,
    ) -> eyre::Result<()> {
        'retry: loop {
            eyre::ensure!(!to_stream_rx.is_closed(), "stream recipient dropped");

            // Connect (with handshake) using retry/backoff logic.
            let mut stream = self.connect_with_retry(&mut from_stream_tx).await?;
            tracing::info!("Connected to {}", self.client.stream.url());

            loop {
                let stream_next = pin!(stream.next());
                let to_stream_rx_next = pin!(to_stream_rx.recv());
                match select(to_stream_rx_next, stream_next).await {
                    future::Either::Left((request, _)) => {
                        let Some(request) = request else {
                            continue 'retry;
                        };

                        let Ok(()) = stream.send(request).await else {
                            continue 'retry;
                        };
                    }
                    future::Either::Right((message, _)) => {
                        let Some(message) = message else {
                            continue 'retry;
                        };

                        match message {
                            Ok(message) => {
                                let message = self.processor.process_message(message);
                                let Ok(()) = from_stream_tx.send(message).await else {
                                    continue 'retry;
                                };
                            }
                            Err(err) => tracing::warn!(?err, "reading message error"),
                        }
                    }
                }
            }
        }
    }

    /// Attempt to connect and perform a handshake using exponential backoff.
    async fn connect_with_retry(
        &mut self,
        from_stream_tx: &mut Sender<T::Output>,
    ) -> eyre::Result<Framed<tokio_rustls::client::TlsStream<TcpStream>, StreamAPIClientCodec>>
    {
        loop {
            let server_addr = self.client.stream.url();
            let host = server_addr
                .host_str()
                .ok_or_else(|| eyre::eyre!("invalid betfair url"))?;
            let port = server_addr.port().unwrap_or(443);

            let domain_str = server_addr
                .domain()
                .ok_or_else(|| eyre::eyre!("domain must be known"))?;
            let domain = rustls::pki_types::ServerName::try_from(domain_str.to_owned())
                .wrap_err("failed to parse server name")?;

            let mut backoff = ExponentialBuilder::new().build();

            let mut delay = async move || {
                if let Some(delay) = backoff.next() {
                    sleep(delay).await;
                    Ok(())
                } else {
                    eyre::bail!("exceeded retry attempts, could not connect");
                }
            };
            // Resolve socket addresses each iteration in case DNS changes
            let Some(socket_addr) = tokio::net::lookup_host((host, port)).await?.next() else {
                eyre::bail!("no valid socket addresses for {host}:{port}")
            };

            let tcp_stream = TcpStream::connect(socket_addr).await;
            let Ok(stream) = tcp_stream else {
                tracing::error!(err = ?tcp_stream.unwrap_err(), "Connect error. Retrying...");
                delay().await?;
                continue;
            };
            let tls_stream = tls_connector()?.connect(domain.clone(), stream).await?;
            let mut tls_stream = Framed::new(tls_stream, StreamAPIClientCodec);

            let handshake = self.handshake(from_stream_tx, &mut tls_stream).await;
            match handshake {
                Err(e) => {
                    tracing::error!(?e, "Handshake error. Retrying...");
                    delay().await?;
                    continue;
                }
                Ok(status) => match status {
                    StreamAction::Continue => {
                        return Ok(tls_stream);
                    }
                    StreamAction::WaitAndRetry => {
                        delay().await?;
                        continue;
                    }
                    StreamAction::Reauthenticate => {
                        self.client.update_auth_token().await?;
                        delay().await?;
                        continue;
                    }
                    StreamAction::Fatal => eyre::bail!("fatal error in stream processing"),
                },
            }
        }
    }

    async fn handshake(
        &mut self,
        from_stream_tx: &mut Sender<T::Output>,
        stream: &mut Framed<tokio_rustls::client::TlsStream<TcpStream>, StreamAPIClientCodec>,
    ) -> eyre::Result<StreamAction> {
        // await con message
        let res = stream.next().await.ok_or_eyre("steam exited")??;
        tracing::info!(?res, "message from stream");
        let ResponseMessage::Connection(_) = &res else {
            eyre::bail!("straem responded with invalid connection message")
        };
        let message = self.processor.process_message(res);
        from_stream_tx.send(message.clone()).await?;

        // send auth msg
        let msg = authentication_message::AuthenticationMessage {
            id: Some(-1),
            session: self.client.session_token().0.expose_secret().clone(),
            app_key: self
                .client
                .secret_provider
                .application_key
                .0
                .expose_secret()
                .clone(),
        };
        stream.send(RequestMessage::Authentication(msg)).await?;

        // await status message
        let message = stream.next().await.ok_or_eyre("steam exited")??;
        tracing::info!(?message, "message from stream");
        let ResponseMessage::Status(status_message) = &message else {
            eyre::bail!("straem responded with invalid status message")
        };
        let processed_message = self.processor.process_message(message.clone());
        from_stream_tx.send(processed_message).await?;

        let Err(err) = &status_message.0 else {
            return Ok(StreamAction::Continue);
        };

        tracing::error!(?err, "stream respondend wit an error");
        let action = match err.error_code {
            ErrorCode::NoAppKey => StreamAction::Fatal,
            ErrorCode::InvalidAppKey => StreamAction::Fatal,
            ErrorCode::NoSession => StreamAction::Reauthenticate,
            ErrorCode::InvalidSessionInformation => StreamAction::Reauthenticate,
            ErrorCode::NotAuthorized => StreamAction::Reauthenticate,
            ErrorCode::InvalidInput => StreamAction::Fatal,
            ErrorCode::InvalidClock => StreamAction::Fatal,
            ErrorCode::UnexpectedError => StreamAction::Fatal,
            ErrorCode::Timeout => StreamAction::WaitAndRetry,
            ErrorCode::SubscriptionLimitExceeded => StreamAction::WaitAndRetry,
            ErrorCode::InvalidRequest => StreamAction::Fatal,
            ErrorCode::ConnectionFailed => StreamAction::WaitAndRetry,
            ErrorCode::MaxConnectionLimitExceeded => StreamAction::Fatal,
            ErrorCode::TooManyRequests => StreamAction::WaitAndRetry,
        };

        Ok(action)
    }
}

enum StreamAction {
    Continue,
    WaitAndRetry,
    Reauthenticate,
    Fatal,
}

#[tracing::instrument(err)]
fn tls_connector() -> eyre::Result<tokio_rustls::TlsConnector> {
    use tokio_rustls::TlsConnector;

    let mut roots = rustls::RootCertStore::empty();
    let native_certs = rustls_native_certs::load_native_certs();
    for cert in native_certs.certs {
        roots.add(cert)?;
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(TlsConnector::from(Arc::new(config)))
}

/// Defines the encoding and decoding of Betfair stream api data structures using tokio
pub struct StreamAPIClientCodec;

impl Decoder for StreamAPIClientCodec {
    type Item = ResponseMessage;
    type Error = eyre::Report;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Find position of `\n` first
        if let Some(pos) = src.iter().position(|&byte| byte == b'\n') {
            // Check if the preceding byte is `\r`
            let delimiter_size = if pos > 0 && src[pos - 1] == b'\r' {
                2
            } else {
                1
            };

            // Extract up to and including the delimiter
            let line = src.split_to(pos + 1);

            // Separate out the delimiter bytes
            let (json_part, _) = line.split_at(line.len().saturating_sub(delimiter_size));

            // Now we can parse it as JSON
            let data = serde_json::from_slice::<Self::Item>(json_part)?;
            return Ok(Some(data));
        }
        Ok(None)
    }
}

impl Encoder<RequestMessage> for StreamAPIClientCodec {
    type Error = eyre::Report;

    fn encode(
        &mut self,
        item: RequestMessage,
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

#[cfg(test)]
mod tests {

    use core::fmt::Write;

    use super::*;

    #[tokio::test]
    async fn can_resolve_host_ipv4() {
        let url = url::Url::parse("tcptls://stream-api.betfair.com:443").unwrap();
        let host = url.host_str().unwrap();
        let port = url
            .port()
            .unwrap_or_else(|| if url.scheme() == "https" { 443 } else { 80 });
        let socket_addr = tokio::net::lookup_host((host, port))
            .await
            .unwrap()
            .next()
            .unwrap();
        assert!(socket_addr.ip().is_ipv4());
        assert_eq!(socket_addr.port(), 443);
    }

    #[test]
    fn can_decode_single_message() {
        let msg = r#"{"op":"connection","connectionId":"002-051134157842-432409"}"#;
        let separator = "\r\n";
        let data = format!("{msg}{separator}");

        let mut codec = StreamAPIClientCodec;
        let mut buf = bytes::BytesMut::from(data.as_bytes());
        let msg = codec.decode(&mut buf).unwrap().unwrap();

        assert!(matches!(msg, ResponseMessage::Connection(_)));
    }

    #[test]
    fn can_decode_multiple_messages() {
        // contains two messages
        let msg_one = r#"{"op":"connection","connectionId":"002-051134157842-432409"}"#;
        let msg_two = r#"{"op":"ocm","id":3,"clk":"AAAAAAAA","status":503,"pt":1498137379766,"ct":"HEARTBEAT"}"#;
        let separator = "\r\n";
        let data = format!("{msg_one}{separator}{msg_two}{separator}");

        let mut codec = StreamAPIClientCodec;
        let mut buf = bytes::BytesMut::from(data.as_bytes());
        let msg_one = codec.decode(&mut buf).unwrap().unwrap();
        let msg_two = codec.decode(&mut buf).unwrap().unwrap();

        assert!(matches!(msg_one, ResponseMessage::Connection(_)));
        assert!(matches!(msg_two, ResponseMessage::OrderChange(_)));
    }

    #[test]
    fn can_decode_multiple_partial_messages() {
        // contains two messages
        let msg_one = r#"{"op":"connection","connectionId":"002-051134157842-432409"}"#;
        let msg_two_pt_one = r#"{"op":"ocm","id":3,"clk""#;
        let msg_two_pt_two = r#":"AAAAAAAA","status":503,"pt":1498137379766,"ct":"HEARTBEAT"}"#;
        let separator = "\r\n";
        let data = format!("{msg_one}{separator}{msg_two_pt_one}");

        let mut codec = StreamAPIClientCodec;
        let mut buf = bytes::BytesMut::from(data.as_bytes());
        let msg_one = codec.decode(&mut buf).unwrap().unwrap();
        let msg_two_attempt = codec.decode(&mut buf).unwrap();
        assert!(msg_two_attempt.is_none());
        buf.write_str(msg_two_pt_two).unwrap();
        buf.write_str(separator).unwrap();
        let msg_two = codec.decode(&mut buf).unwrap().unwrap();

        assert!(matches!(msg_one, ResponseMessage::Connection(_)));
        assert!(matches!(msg_two, ResponseMessage::OrderChange(_)));
    }

    #[test]
    fn can_decode_subsequent_messages() {
        // contains two messages
        let msg_one = r#"{"op":"connection","connectionId":"002-051134157842-432409"}"#;
        let msg_two = r#"{"op":"ocm","id":3,"clk":"AAAAAAAA","status":503,"pt":1498137379766,"ct":"HEARTBEAT"}"#;
        let separator = "\r\n";
        let data = format!("{msg_one}{separator}");

        let mut codec = StreamAPIClientCodec;
        let mut buf = bytes::BytesMut::from(data.as_bytes());
        let msg_one = codec.decode(&mut buf).unwrap().unwrap();
        let msg_two_attempt = codec.decode(&mut buf).unwrap();
        assert!(msg_two_attempt.is_none());
        let data = format!("{msg_two}{separator}");
        buf.write_str(data.as_str()).unwrap();
        let msg_two = codec.decode(&mut buf).unwrap().unwrap();

        assert!(matches!(msg_one, ResponseMessage::Connection(_)));
        assert!(matches!(msg_two, ResponseMessage::OrderChange(_)));
    }

    #[test]
    fn can_encode_message() {
        let msg = RequestMessage::Authentication(
            betfair_stream_types::request::authentication_message::AuthenticationMessage {
                id: Some(1),
                session: "sss".to_owned(),
                app_key: "aaaa".to_owned(),
            },
        );
        let mut codec = StreamAPIClientCodec;
        let mut buf = bytes::BytesMut::new();
        codec.encode(msg, &mut buf).unwrap();

        let data = buf.freeze();
        let data = core::str::from_utf8(&data).unwrap();

        // assert that we have the suffix \r\n
        assert!(data.ends_with("\r\n"));
        // assert that we have the prefix {"op":"authentication"
        assert!(data.starts_with("{\"op\":\"authentication\""));
    }
}

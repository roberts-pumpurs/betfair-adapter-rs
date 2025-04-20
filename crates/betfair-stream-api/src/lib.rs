//! Betfair Stream API client implementation.
//!
//! This crate provides an asynchronous client for interacting with Betfair's Streaming API.
//! It manages connection setup, handshake, framing, heartbeats, and automatic reconnections.
//! Users can customize how incoming messages are handled by implementing the `MessageProcessor` trait
//! or using the built-in `Cache` processor for maintaining market and order caches.
extern crate alloc;
pub mod cache;
use backon::{BackoffBuilder as _, ExponentialBuilder};
use betfair_adapter::{Authenticated, BetfairRpcClient};
pub use betfair_stream_types as types;
use betfair_stream_types::{
    request::{RequestMessage, authentication_message, heartbeat_message::HeartbeatMessage},
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
use core::fmt;
use core::{pin::pin, time::Duration};
use eyre::Context as _;
use futures::{
    SinkExt as _, StreamExt as _,
    future::{self, select},
};
use std::sync::Arc;
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
    time::sleep,
};
use tokio_stream::wrappers::{IntervalStream, ReceiverStream};
use tokio_util::{
    bytes,
    codec::{Decoder, Encoder, Framed},
};

/// A Betfair Stream API client that handles connection, handshake, incoming/outgoing messages,
/// heartbeat and automatic reconnects.
/// Builder for creating a Betfair Streaming API client.
///
/// # Type Parameters
///
/// - `T`: A type that implements `MessageProcessor`, used to handle incoming `ResponseMessage` objects.
pub struct BetfairStreamBuilder<T: MessageProcessor> {
    /// betfair cient
    pub client: BetfairRpcClient<Authenticated>,
    /// Heartbeat interval (used only if heartbeat_enabled is true)
    pub heartbeat_interval: Option<Duration>,
    /// The intermediate processor of messages
    pub processor: T,
}

/// Handle to a running Betfair Streaming API client.
///
/// Provides channels to send requests (`send_to_stream`) and receive processed messages (`sink`).
pub struct BetfairStreamClient<T: MessageProcessor> {
    /// send a message to the Betfair stream
    pub send_to_stream: Sender<RequestMessage>,
    /// Receive a message from the stream
    pub sink: Receiver<T::Output>,
}

/// Default `MessageProcessor` implementation that maintains market and order caches.
///
/// It updates an internal `StreamState` to apply incremental updates to market and order books.
pub struct Cache {
    state: StreamState,
}

/// Variants of messages produced by the cache-based processor.
///
/// `CachedMessage` represents high-level events derived from raw Betfair streaming responses,
/// enriched with internal cache state for market and order books.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CachedMessage {
    /// A connection handshake message received from the stream,
    /// containing connection ID and related metadata.
    /// Also returned on heartbeat messages.
    Connection(ConnectionMessage),

    /// A batch of market book updates, each describing the current state or changes of a market.
    MarketChange(Vec<MarketBookCache>),

    /// A batch of order book updates, representing new orders, matched orders,
    /// and cancellations in the order cache.
    OrderChange(Vec<OrderBookCache>),

    /// A status message from the stream, used for heartbeats,
    /// subscription confirmations, or error notifications.
    Status(StatusMessage),
}

impl MessageProcessor for Cache {
    type Output = CachedMessage;

    fn process_message(&mut self, message: ResponseMessage) -> Option<Self::Output> {
        match message {
            ResponseMessage::Connection(connection_message) => {
                Some(CachedMessage::Connection(connection_message))
            }
            ResponseMessage::MarketChange(market_change_message) => {
                let data = self
                    .state
                    .market_change_update(market_change_message)
                    .map(|markets| markets.into_iter().cloned().collect::<Vec<_>>())
                    .map(CachedMessage::MarketChange);

                data
            }
            ResponseMessage::OrderChange(order_change_message) => {
                let data = self
                    .state
                    .order_change_update(order_change_message)
                    .map(|markets| markets.into_iter().cloned().collect::<Vec<_>>())
                    .map(CachedMessage::OrderChange);

                data
            }
            ResponseMessage::Status(status_message) => Some(CachedMessage::Status(status_message)),
        }
    }
}

/// `MessageProcessor` that forwards raw `ResponseMessage` objects without transformation.
pub struct Forwarder;
impl MessageProcessor for Forwarder {
    type Output = ResponseMessage;

    fn process_message(&mut self, message: ResponseMessage) -> Option<Self::Output> {
        Some(message)
    }
}
/// Trait for processing incoming Betfair streaming `ResponseMessage` objects into user-defined outputs.
///
/// Implementers can filter or transform messages and control which messages are forwarded to the client sink.
pub trait MessageProcessor: Send + Sync + 'static {
    /// The processed message type produced by `process_message`
    type Output: Send + Clone + Sync + 'static + core::fmt::Debug;

    /// Process an incoming `ResponseMessage`.
    ///
    /// Returns `Some(Output)` to forward a processed message, or `None` to drop it.
    fn process_message(&mut self, message: ResponseMessage) -> Option<Self::Output>;
}

impl<T: MessageProcessor> BetfairStreamBuilder<T> {
    /// Creates a new `BetfairStreamBuilder` with the given authenticated RPC client.
    ///
    /// Uses the default `Cache` message processor to maintain market and order caches.
    /// By default, no heartbeat messages are sent.
    ///
    /// # Parameters
    ///
    /// * `client` - An authenticated Betfair RPC client for establishing the streaming connection.
    ///
    /// # Returns
    ///
    /// A `BetfairStreamBuilder` configured with cache-based message processing.
    pub fn new(client: BetfairRpcClient<Authenticated>) -> BetfairStreamBuilder<Cache> {
        BetfairStreamBuilder {
            client,
            heartbeat_interval: None,
            processor: Cache {
                state: StreamState::new(),
            },
        }
    }

    /// Creates a new `BetfairStreamBuilder` with raw message forwarding.
    ///
    /// Uses the `Forwarder` message processor to forward raw `ResponseMessage` objects without caching.
    /// By default, no heartbeat messages are sent.
    ///
    /// # Parameters
    ///
    /// * `client` - An authenticated Betfair RPC client for establishing the streaming connection.
    ///
    /// # Returns
    ///
    /// A `BetfairStreamBuilder` configured to forward raw messages.
    pub fn new_without_cache(
        client: BetfairRpcClient<Authenticated>,
    ) -> BetfairStreamBuilder<Forwarder> {
        BetfairStreamBuilder {
            client,
            heartbeat_interval: None,
            processor: Forwarder,
        }
    }

    /// Enables periodic heartbeat messages to keep the streaming connection alive.
    ///
    /// # Parameters
    ///
    /// * `interval` - The duration between heartbeat messages.
    ///
    /// # Returns
    ///
    /// The updated `BetfairStreamBuilder` with heartbeat enabled.
    pub fn with_heartbeat(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = Some(interval);
        self
    }

    /// Starts the Betfair streaming client and returns handles for interaction.
    ///
    /// This will spawn an asynchronous task that manages the connection, handshake,
    /// incoming/outgoing messages, heartbeats (if enabled), and automatic reconnections.
    ///
    /// # Returns
    ///
    /// * `BetfairStreamClient<T>` - A client handle providing:
    ///     - `send_to_stream`: a channel sender for outgoing `RequestMessage`s.
    ///     - `sink`: a channel receiver for processed messages of type `T::Output`.
    /// * `JoinHandle<eyre::Result<()>>` - A handle to the background task driving the streaming logic.
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

    async fn run(
        self,
        from_stream_tx: Sender<T::Output>,
        to_stream_rx: Receiver<RequestMessage>,
    ) -> eyre::Result<()> {
        if let Some(hb) = self.heartbeat_interval {
            let heartbeat_stream = {
                let mut interval = tokio::time::interval(hb);
                interval.reset();
                let interval_stream = IntervalStream::new(interval).fuse();
                interval_stream
                    .map(move |instant| HeartbeatMessage {
                        id: Some(
                            instant
                                .into_std()
                                .elapsed()
                                .as_secs()
                                .try_into()
                                .unwrap_or_default(),
                        ),
                    })
                    .map(RequestMessage::Heartbeat)
                    .boxed()
            };
            let input_stream = futures::stream::select_all([
                heartbeat_stream,
                ReceiverStream::new(to_stream_rx).boxed(),
            ]);

            self.run_base(from_stream_tx, input_stream).await
        } else {
            self.run_base(from_stream_tx, ReceiverStream::new(to_stream_rx))
                .await
        }
    }

    async fn run_base(
        mut self,
        mut from_stream_tx: Sender<T::Output>,
        mut to_stream_rx: impl futures::Stream<Item = RequestMessage> + Unpin,
    ) -> eyre::Result<()> {
        let mut backoff = ExponentialBuilder::new().build();
        'retry: loop {
            // add exponential recovery
            let Some(delay) = backoff.next() else {
                eyre::bail!("connection retry attempts exceeded")
            };
            sleep(delay).await;

            // Connect (with handshake) using retry logic.
            let mut stream = self.connect_with_retry(&mut from_stream_tx).await?;
            tracing::info!("Connected to {}", self.client.stream.url());

            loop {
                let stream_next = pin!(stream.next());
                let to_stream_rx_next = pin!(to_stream_rx.next());
                match select(to_stream_rx_next, stream_next).await {
                    future::Either::Left((request, _)) => {
                        let Some(request) = request else {
                            tracing::warn!("request returned None");
                            continue 'retry;
                        };

                        tracing::debug!(?request, "sending to betfair");
                        let Ok(()) = stream.send(request).await else {
                            tracing::warn!("could not send request to stream");
                            continue 'retry;
                        };
                    }
                    future::Either::Right((message, _)) => {
                        let Some(message) = message else {
                            tracing::warn!("stream returned None");
                            continue 'retry;
                        };

                        match message {
                            Ok(message) => {
                                let message = self.processor.process_message(message);
                                tracing::debug!(?message, "received from betfair");
                                let Some(message) = message else {
                                    continue;
                                };

                                let Ok(()) = from_stream_tx.send(message).await else {
                                    tracing::warn!("could not send stream message to sink");
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
    #[tracing::instrument(skip_all, err)]
    async fn connect_with_retry(
        &mut self,
        from_stream_tx: &mut Sender<T::Output>,
    ) -> eyre::Result<Framed<tokio_rustls::client::TlsStream<TcpStream>, StreamAPIClientCodec>>
    {
        let mut backoff = ExponentialBuilder::new().build();
        let mut delay = async || {
            if let Some(delay) = backoff.next() {
                sleep(delay).await;
                Ok(())
            } else {
                eyre::bail!("exceeded retry attempts, could not connect");
            }
        };

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

            match self.handshake(from_stream_tx, &mut tls_stream).await {
                Ok(()) => return Ok(tls_stream),
                Err(err) => match err {
                    HandshakeErr::WaitAndRetry => {
                        delay().await?;
                        continue;
                    }
                    HandshakeErr::Reauthenticate => {
                        self.client.update_auth_token().await?;
                        delay().await?;
                        continue;
                    }
                    HandshakeErr::Fatal => eyre::bail!("fatal error in stream processing"),
                },
            }
        }
    }

    #[tracing::instrument(err, skip_all)]
    async fn handshake(
        &mut self,
        from_stream_tx: &mut Sender<T::Output>,
        stream: &mut Framed<tokio_rustls::client::TlsStream<TcpStream>, StreamAPIClientCodec>,
    ) -> Result<(), HandshakeErr> {
        // await con message
        let res = stream
            .next()
            .await
            .transpose()
            .inspect_err(|err| {
                tracing::warn!(?err, "error when parsing stream message");
            })
            .map_err(|_| HandshakeErr::WaitAndRetry)?
            .ok_or(HandshakeErr::WaitAndRetry)?;
        tracing::info!(?res, "message from stream");
        let message = self
            .processor
            .process_message(res.clone())
            .ok_or(HandshakeErr::Fatal)
            .inspect_err(|err| tracing::error!(?err))?;
        from_stream_tx
            .send(message.clone())
            .await
            .inspect_err(|err| tracing::warn!(?err))
            .map_err(|_| HandshakeErr::Fatal)?;
        let ResponseMessage::Connection(_) = &res else {
            tracing::warn!("stream responded with invalid connection message");
            return Err(HandshakeErr::Reauthenticate);
        };

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
        stream
            .send(RequestMessage::Authentication(msg))
            .await
            .inspect_err(|err| tracing::warn!(?err, "stream exited"))
            .map_err(|_| HandshakeErr::WaitAndRetry)?;

        // await status message
        let message = stream
            .next()
            .await
            .transpose()
            .inspect_err(|err| {
                tracing::warn!(?err, "error when parsing stream message");
            })
            .map_err(|_| HandshakeErr::WaitAndRetry)?
            .ok_or(HandshakeErr::WaitAndRetry)?;
        let processed_message = self
            .processor
            .process_message(message.clone())
            .ok_or(HandshakeErr::Fatal)
            .inspect_err(|err| tracing::warn!(?err))
            .map_err(|_| HandshakeErr::Fatal)?;
        from_stream_tx
            .send(processed_message)
            .await
            .inspect_err(|err| tracing::warn!(?err))
            .map_err(|_| HandshakeErr::Fatal)?;
        tracing::info!(?message, "message from stream");
        let ResponseMessage::Status(status_message) = &message else {
            tracing::warn!("expected status message, got {message:?}");
            return Err(HandshakeErr::WaitAndRetry);
        };

        let StatusMessage::Failure(err) = &status_message else {
            return Ok(());
        };

        tracing::error!(?err, "stream respondend wit an error");
        let action = match err.error_code {
            ErrorCode::NoAppKey => HandshakeErr::Fatal,
            ErrorCode::InvalidAppKey => HandshakeErr::Fatal,
            ErrorCode::NoSession => HandshakeErr::Reauthenticate,
            ErrorCode::InvalidSessionInformation => HandshakeErr::Reauthenticate,
            ErrorCode::NotAuthorized => HandshakeErr::Reauthenticate,
            ErrorCode::InvalidInput => HandshakeErr::Fatal,
            ErrorCode::InvalidClock => HandshakeErr::Fatal,
            ErrorCode::UnexpectedError => HandshakeErr::Fatal,
            ErrorCode::Timeout => HandshakeErr::WaitAndRetry,
            ErrorCode::SubscriptionLimitExceeded => HandshakeErr::WaitAndRetry,
            ErrorCode::InvalidRequest => HandshakeErr::Fatal,
            ErrorCode::ConnectionFailed => HandshakeErr::WaitAndRetry,
            ErrorCode::MaxConnectionLimitExceeded => HandshakeErr::Fatal,
            ErrorCode::TooManyRequests => HandshakeErr::WaitAndRetry,
        };

        Err(action)
    }
}

#[derive(Debug)]
enum HandshakeErr {
    WaitAndRetry,
    Reauthenticate,
    Fatal,
}

impl fmt::Display for HandshakeErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Stream Handshake Error {:?}", self)
    }
}

impl core::error::Error for HandshakeErr {}

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

    use core::fmt::Write as _;

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

//! Betfair stream client
use backon::{BackoffBuilder, ExponentialBackoff, ExponentialBuilder};
use betfair_adapter::{ApplicationKey, SessionToken};
use betfair_stream_types::{
    request::{RequestMessage, authentication_message},
    response::{
        ResponseMessage,
        status_message::{StatusCode, StatusMessage},
    },
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
pub struct BetfairStreamConnection {
    /// TCP address of the Betfair server (for example: "betfair.server.com:1234")
    pub server_addr: url::Url,
    /// Heartbeat interval (used only if heartbeat_enabled is true)
    pub heartbeat_interval: Option<Duration>,
    pub session_token: SessionToken,
    pub application_key: ApplicationKey,
}

pub struct BetfairStreamClient {
    pub send_to_stream: Sender<RequestMessage>,
    pub sink: Receiver<ResponseMessage>,
}

impl BetfairStreamConnection {
    pub async fn start(self) -> (BetfairStreamClient, JoinHandle<eyre::Result<()>>) {
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
        self,
        mut from_stream_tx: Sender<ResponseMessage>,
        mut to_stream_rx: Receiver<RequestMessage>,
    ) -> eyre::Result<()> {
        // Connect (with handshake) using retry/backoff logic.
        let mut stream = self.connect_with_retry(&mut from_stream_tx).await?;
        tracing::info!("Connected to {}", self.server_addr);

        loop {
            let stream_next = pin!(stream.next());
            let to_stream_rx_next = pin!(to_stream_rx.recv());
            match select(to_stream_rx_next, stream_next).await {
                future::Either::Left((request, _)) => {
                    if request.is_some() {
                        stream.send(request.unwrap()).await?;
                    }
                }
                future::Either::Right((message, _)) => {
                    if message.is_some() {
                        from_stream_tx.send(message.unwrap().unwrap()).await?;
                    }
                }
            }
        }
    }

    /// Attempt to connect and perform a handshake using exponential backoff.
    async fn connect_with_retry(
        &self,
        from_stream_tx: &mut Sender<ResponseMessage>,
    ) -> eyre::Result<Framed<tokio_rustls::client::TlsStream<TcpStream>, StreamAPIClientCodec>>
    {
        let host = self
            .server_addr
            .host_str()
            .ok_or_else(|| eyre::eyre!("invalid betfair url"))?;
        let port = self.server_addr.port().unwrap_or(443);

        let domain_str = self
            .server_addr
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
        loop {
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
                Ok(_status) => {
                    return Ok(tls_stream);
                }
            }
        }
    }

    /// Perform the handshake by sending a handshake message and waiting for a valid reply.
    /// In this simple example we send `handshake\n` and expect the reply to contain
    /// the substring `handshake_ok`.
    async fn handshake(
        &self,
        from_stream_tx: &mut Sender<ResponseMessage>,
        stream: &mut Framed<tokio_rustls::client::TlsStream<TcpStream>, StreamAPIClientCodec>,
    ) -> eyre::Result<StatusMessage> {
        // await con message
        let res = stream.next().await.ok_or_eyre("steam exited")??;
        tracing::info!(?res, "message from stream");
        from_stream_tx.send(res.clone()).await?;
        let ResponseMessage::Connection(_con_message) = res else {
            eyre::bail!("straem responded with invalid connection message")
        };

        // send auth msg
        let msg = authentication_message::AuthenticationMessage {
            id: Some(-1),
            session: self.session_token.0.expose_secret().clone(),
            app_key: self.application_key.0.expose_secret().clone(),
        };
        stream.send(RequestMessage::Authentication(msg)).await?;

        // await status message
        let res = stream.next().await.ok_or_eyre("steam exited")??;
        tracing::info!(?res, "message from stream");
        from_stream_tx.send(res.clone()).await?;
        let ResponseMessage::Status(status_message) = res else {
            eyre::bail!("straem responded with invalid status message")
        };

        // ensure that the authentication was successful
        eyre::ensure!(
            status_message.status_code == Some(StatusCode::Success),
            "authentication was not successful"
        );

        Ok(status_message)
    }
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

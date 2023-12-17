use std::borrow::Cow;
use std::net::SocketAddr;

use betfair_adapter::{ApplicationKey, BetfairUrl, SessionToken};
use betfair_stream_types::{authentication_message, RequestMessage, ResponseMessage};
use futures_util::sink::SinkExt;
use futures_util::Future;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::bytes;
use tokio_util::codec::{Decoder, Encoder, FramedRead, FramedWrite};

#[derive(Debug)]
pub struct BetfairStreamAPI<'a> {
    url: BetfairUrl<'a, betfair_adapter::Stream, (url::Url, SocketAddr)>,
    application_key: Cow<'a, ApplicationKey>,
    session_token: Cow<'a, SessionToken>,
    state: StreamState,
}

impl<'a> BetfairStreamAPI<'a> {
    pub fn new(
        url: BetfairUrl<'a, betfair_adapter::Stream, (url::Url, SocketAddr)>,
        application_key: Cow<'a, ApplicationKey>,
        session_token: Cow<'a, SessionToken>,
    ) -> Self {
        Self {
            url,
            application_key,
            session_token,
            state: StreamState::PreAuth,
        }
    }

    pub async fn connect_tls(
        &mut self,
    ) -> Result<
        (
            impl Future<Output = Result<(), StreamError>>,
            impl futures_util::Stream<Item = Result<ResponseMessage, StreamError>>,
        ),
        StreamError,
    > {
        let (url, socket) = self.url.url().clone();
        let domain = url.domain().unwrap().to_string();
        let domain = rustls::pki_types::ServerName::try_from(domain).unwrap();
        let stream = TcpStream::connect(&socket).await?;
        let connector = tls_connector();
        let stream = connector.connect(domain, stream).await.unwrap();
        let (read, write) = tokio::io::split(stream);
        self.process(write, read, futures_util::stream::empty())
            .await
    }

    pub async fn connect(
        &mut self,
    ) -> Result<
        (
            impl Future<Output = Result<(), StreamError>>,
            impl futures_util::Stream<Item = Result<ResponseMessage, StreamError>>,
        ),
        StreamError,
    > {
        let (_url, socket) = self.url.url().clone();
        let stream = TcpStream::connect(&socket).await?;

        let (read, write) = stream.into_split();
        self.process(write, read, futures_util::stream::empty())
            .await
    }

    async fn process<
        I: AsyncWrite + std::fmt::Debug + Send + Unpin,
        O: AsyncRead + std::fmt::Debug + Send + Unpin,
    >(
        &mut self,
        input: I,
        output: O,
        mut incoming_commands: impl futures_util::Stream<Item = RequestMessage>
            + std::marker::Send
            + std::marker::Unpin,
    ) -> Result<
        (
            impl Future<Output = Result<(), StreamError>>,
            impl futures_util::Stream<Item = Result<ResponseMessage, StreamError>>,
        ),
        StreamError,
    > {
        let mut rng = SmallRng::from_entropy();
        let mut writer = FramedWrite::new(input, StreamAPICodec);
        let mut reader = FramedRead::new(output, StreamAPICodec);

        loop {
            match &mut self.state {
                StreamState::PreAuth => {
                    let data = next_msg(&mut reader).await?;
                    if let ResponseMessage::Connection(connection_message) = data {
                        if let Some(connection_id) = connection_message.connection_id {
                            tracing::debug!(connection_id = ?connection_id, "Connection established");
                            self.state = StreamState::SendAuth { connection_id };
                            continue
                        }
                        tracing::error!("No connection id");
                        return Err(StreamError::ConnectionIdNotPresent)
                    }
                    tracing::error!("Unexpected response");
                    return Err(StreamError::UnexpectedResponse(format!("{:?}", data)))
                }
                StreamState::SendAuth { connection_id } => {
                    let id = rng.gen();
                    let authorization_message = RequestMessage::Authentication(
                        authentication_message::AuthenticationMessage {
                            id: Some(id),
                            session: self.session_token.0.expose_secret().clone(),
                            app_key: self.application_key.0.expose_secret().clone(),
                        },
                    );
                    writer.send(authorization_message).await?;
                    self.state = StreamState::AwaitStatus {
                        connection_id: connection_id.clone(),
                    };
                }
                StreamState::AwaitStatus { connection_id } => {
                    let data = next_msg(&mut reader).await?;
                    if let ResponseMessage::StatusMessage(ref status_message) = data {
                        if status_message.connection_id.as_ref() == Some(connection_id) {
                            tracing::debug!(connection_id = ?connection_id, "Authenticated");
                            self.state = StreamState::Authenticated {
                                connection_id: connection_id.clone(),
                            };
                            continue
                        }
                    }
                    tracing::error!("Unexpected response");
                    return Err(StreamError::UnexpectedResponse(format!("{:?}", data)))
                }
                StreamState::Authenticated { connection_id: _ } => break,
            }
        }
        let read_task = async move {
            while let Some(command) = incoming_commands.next().await {
                let _ = writer.send(command).await;
            }
            Ok(())
        };
        let write_task = async_stream::stream! {
            while let Some(data) = reader.next().await {
                yield data;
            }
        };

        Ok((read_task, write_task))
    }
}

async fn next_msg<O: AsyncRead + std::fmt::Debug + Send + Unpin>(
    reader: &mut FramedRead<O, StreamAPICodec>,
) -> Result<ResponseMessage, StreamError> {
    let data = reader
        .next()
        .await
        .transpose()?
        .ok_or_else(|| StreamError::NoData)?;
    Ok(data)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StreamState {
    PreAuth,
    SendAuth { connection_id: String },
    AwaitStatus { connection_id: String },
    Authenticated { connection_id: String },
}

pub struct StreamAPICodec;

impl Decoder for StreamAPICodec {
    type Item = betfair_stream_types::ResponseMessage;
    type Error = StreamError;

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

impl Encoder<betfair_stream_types::RequestMessage> for StreamAPICodec {
    type Error = StreamError;

    fn encode(
        &mut self,
        item: betfair_stream_types::RequestMessage,
        dst: &mut bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        // Serialize the item to a JSON string
        let json = serde_json::to_string(&item)?;
        // Write the JSON string to the buffer, followed by a newline
        dst.extend_from_slice(json.as_bytes());
        dst.extend_from_slice(b"\n");
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error("No data")]
    NoData,
    #[error("Unexpected response {0}")]
    UnexpectedResponse(String),
    #[error("Connection ID not present")]
    ConnectionIdNotPresent,
}

fn tls_connector() -> tokio_rustls::TlsConnector {
    use tokio_rustls::TlsConnector;
    let mut roots = rustls::RootCertStore::empty();
    rustls_native_certs::load_native_certs()
        .expect("could not load platform certs")
        .into_iter()
        .for_each(|cert| {
            roots.add(cert).unwrap();
        });

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    TlsConnector::from(std::sync::Arc::new(config))
}

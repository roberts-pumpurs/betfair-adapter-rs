use std::borrow::Cow;
use std::net::SocketAddr;

use betfair_adapter::{ApplicationKey, SessionToken};
use betfair_stream_types::request::{authentication_message, RequestMessage};
use betfair_stream_types::response::ResponseMessage;
use futures_util::sink::SinkExt;
use futures_util::{Future, StreamExt};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_util::bytes;
use tokio_util::codec::{Decoder, Encoder, FramedRead, FramedWrite};

use crate::StreamError;

#[derive(Debug)]
pub(crate) struct RawStream<'a> {
    application_key: Cow<'a, ApplicationKey>,
    session_token: Cow<'a, SessionToken>,
    state: StreamState,
}

impl<'a> RawStream<'a> {
    pub fn new(
        application_key: Cow<'a, ApplicationKey>,
        session_token: Cow<'a, SessionToken>,
    ) -> Self {
        Self {
            application_key,
            session_token,
            state: StreamState::PreAuth,
        }
    }

    pub async fn connect_tls(
        &mut self,
        domain: &str,
        socket_addr: SocketAddr,
        incoming_commands: impl futures_util::Stream<Item = RequestMessage>
            + std::marker::Send
            + std::marker::Unpin,
    ) -> Result<
        (
            impl Future<Output = Result<(), StreamError>>,
            impl futures_util::Stream<Item = Result<ResponseMessage, StreamError>>,
        ),
        StreamError,
    > {
        let domain = rustls::pki_types::ServerName::try_from(domain.to_string()).unwrap();
        let stream = TcpStream::connect(&socket_addr).await?;
        let connector = tls_connector();
        let stream = connector.connect(domain, stream).await.unwrap();
        let (read, write) = tokio::io::split(stream);
        let (write_to_wire, output) = self.process(write, read, incoming_commands).await?;
        Ok((write_to_wire, output))
    }

    pub async fn connect_non_tls(
        &mut self,
        socket_addr: SocketAddr,
        incoming_commands: impl futures_util::Stream<Item = RequestMessage>
            + std::marker::Send
            + std::marker::Unpin,
    ) -> Result<
        (
            impl Future<Output = Result<(), StreamError>>,
            impl futures_util::Stream<Item = Result<ResponseMessage, StreamError>>,
        ),
        StreamError,
    > {
        let stream = TcpStream::connect(&socket_addr).await.unwrap();
        let (read, write) = stream.into_split();
        let (write_to_wire, output) = self.process(write, read, incoming_commands).await.unwrap();
        Ok((write_to_wire, output))
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
        let mut writer = FramedWrite::new(input, StreamAPIClientCodec);
        let mut reader = FramedRead::new(output, StreamAPIClientCodec);

        self.handshake(&mut reader, &mut writer).await?;
        // read commands and send them to the writer
        let write_task = async move {
            loop {
                tracing::info!("Waiting for command");
                let command = incoming_commands.next().await;
                tracing::info!(command = ?command, "Sending command");
                match command {
                    Some(command) => {
                        let _ = writer.send(command).await;
                    }
                    None => break,
                }
            }
            tracing::error!("Done sending commands");
            Err(StreamError::StreamProcessorMalfunction)
        };

        Ok((
            async move {
                let res = write_task.await.unwrap();
                tracing::error!("Write task finished");
                res
            },
            reader,
        ))
    }

    #[tracing::instrument(level = "DEBUG", skip(self, reader, writer), ret, err)]
    async fn handshake<
        I: AsyncWrite + std::fmt::Debug + Send + Unpin,
        O: AsyncRead + std::fmt::Debug + Send + Unpin,
    >(
        &mut self,
        reader: &mut FramedRead<O, StreamAPIClientCodec>,
        writer: &mut FramedWrite<I, StreamAPIClientCodec>,
    ) -> Result<(), StreamError> {
        let mut rng = SmallRng::from_entropy();
        loop {
            match &mut self.state {
                StreamState::PreAuth => {
                    let data = next_msg(reader).await?;
                    let Some(data) = data else { continue };
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
                    let data = next_msg(reader).await?;
                    let Some(data) = data else { continue };
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
                StreamState::Authenticated { connection_id } => {
                    tracing::info!(connection_id =? connection_id, "Handshake complete");
                    break;
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StreamState {
    PreAuth,
    SendAuth { connection_id: String },
    AwaitStatus { connection_id: String },
    Authenticated { connection_id: String },
}

pub struct StreamAPIClientCodec;

impl Decoder for StreamAPIClientCodec {
    type Item = ResponseMessage;
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

impl Encoder<RequestMessage> for StreamAPIClientCodec {
    type Error = StreamError;

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

async fn next_msg<O: AsyncRead + std::fmt::Debug + Send + Unpin>(
    reader: &mut FramedRead<O, StreamAPIClientCodec>,
) -> Result<Option<ResponseMessage>, StreamError> {
    let data = reader.next().await.transpose()?;
    Ok(data)
}

#[cfg(test)]
mod tests {

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
}

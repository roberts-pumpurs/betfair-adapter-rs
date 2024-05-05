use std::net::SocketAddr;
use std::pin::Pin;

use betfair_adapter::BetfairUrl;
use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::response::ResponseMessage;
use futures_util::sink::SinkExt;
use futures_util::{Future, FutureExt, Stream, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tokio_util::bytes;
use tokio_util::codec::{Decoder, Encoder, FramedRead, FramedWrite};

use crate::StreamError;

pub(crate) async fn connect(
    url: BetfairUrl<betfair_adapter::Stream>,
    command_reader: impl futures_util::Stream<Item = Result<RequestMessage, eyre::Error>>
        + std::marker::Unpin
        + Send
        + 'static,
) -> Result<
    (
        Pin<Box<dyn Future<Output = Result<(), StreamError>> + Send>>,
        Pin<Box<dyn Stream<Item = Result<ResponseMessage, StreamError>> + Send>>,
    ),
    StreamError,
> {
    // TODO get rid of the unwraps
    let host = url.url().host_str().unwrap();
    let is_tls = url.url().scheme() == "https";
    let port = url.url().port().unwrap_or(if is_tls { 443 } else { 80 });
    let socket_addr = tokio::net::lookup_host((host, port)).await.unwrap().next();
    let domain = url.url().domain();
    let result = match (is_tls, domain, socket_addr) {
        (true, Some(domain), Some(socket_addr)) => {
            let (write_to_wire, read) = connect_tls(domain, socket_addr, command_reader).await?;
            Ok((write_to_wire.boxed(), read.boxed()))
        }
        (false, _, Some(socket_addr)) => {
            let (write_to_wire, read) = connect_non_tls(socket_addr, command_reader).await?;
            Ok((write_to_wire.boxed(), read.boxed()))
        }
        _ => Err(StreamError::MisconfiguredStreamURL),
    }?;

    Ok(result)
}

pub(crate) async fn connect_tls(
    domain: &str,
    socket_addr: SocketAddr,
    incoming_commands: impl futures_util::Stream<Item = Result<RequestMessage, eyre::Error>>
        + std::marker::Unpin
        + 'static,
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
    let (write_to_wire, output) = process(write, read, incoming_commands).await?;
    Ok((write_to_wire, output))
}

pub(crate) async fn connect_non_tls(
    socket_addr: SocketAddr,
    incoming_commands: impl futures_util::Stream<Item = Result<RequestMessage, eyre::Error>>
        + std::marker::Unpin
        + 'static,
) -> Result<
    (
        impl Future<Output = Result<(), StreamError>>,
        impl futures_util::Stream<Item = Result<ResponseMessage, StreamError>>,
    ),
    StreamError,
> {
    let stream = TcpStream::connect(&socket_addr).await.unwrap();
    let (read, write) = stream.into_split();
    let (write_to_wire, output) = process(write, read, incoming_commands).await.unwrap();
    Ok((write_to_wire, output))
}

async fn process<
    I: AsyncWrite + std::fmt::Debug + Send + Unpin,
    O: AsyncRead + std::fmt::Debug + Send + Unpin,
>(
    input: I,
    output: O,
    mut incoming_commands: impl futures_util::Stream<Item = Result<RequestMessage, eyre::Error>>
        + std::marker::Unpin
        + 'static,
) -> Result<
    (
        impl Future<Output = Result<(), StreamError>>,
        impl futures_util::Stream<Item = Result<ResponseMessage, StreamError>>,
    ),
    StreamError,
> {
    let mut writer = FramedWrite::new(input, StreamAPIClientCodec);
    let reader = FramedRead::new(output, StreamAPIClientCodec);

    // read commands and send them to the writer
    let write_task = async move {
        loop {
            tracing::trace!("Waiting for command");
            let command = incoming_commands.next().await;
            tracing::info!(command = ?command, "Sending command");
            match command {
                Some(Ok(command)) => {
                    let _ = writer.send(command).await;
                }
                _ => break,
            }
        }
        tracing::error!("Done sending commands");
        Err(StreamError::StreamProcessorMalfunction)
    };

    Ok((
        async move {
            let res = write_task.await;
            tracing::error!("Write task finished");
            res
        },
        reader,
    ))
}
pub(crate) struct StreamAPIClientCodec;

impl Decoder for StreamAPIClientCodec {
    type Item = ResponseMessage;
    type Error = StreamError;

    // todo: write a test that checks the behaviour when we have more than 2 msgs in the source
    // bytes
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        use itertools::*;

        // Check if there is a newline character in the buffer
        if let Some(line) = src
            .iter()
            .tuple_windows::<(_, _)>()
            .position(|(a, b)| a == &b'\r' && b == &b'\n')
            .map(|idx| src.split_to(idx + 2))
        {
            if let Some((json, _delimiters)) = line.split_last_chunk::<2>() {
                // Deserialize the JSON data
                let data = serde_json::from_slice::<Self::Item>(&json)?;
                return Ok(Some(data))
            }

            Ok(None)
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

#[cfg(test)]
mod tests {

    use std::fmt::Write;

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
        let data = format!("{}{}", msg, separator);

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
        let data = format!("{}{}{}{}", msg_one, separator, msg_two, separator);

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
        let data = format!("{}{}{}", msg_one, separator, msg_two_pt_one);

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
        let data = format!("{}{}", msg_one, separator);

        let mut codec = StreamAPIClientCodec;
        let mut buf = bytes::BytesMut::from(data.as_bytes());
        let msg_one = codec.decode(&mut buf).unwrap().unwrap();
        let msg_two_attempt = codec.decode(&mut buf).unwrap();
        assert!(msg_two_attempt.is_none());
        let data = format!("{}{}", msg_two, separator);
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
                session: "sss".to_string(),
                app_key: "aaaa".to_string(),
            },
        );
        let mut codec = StreamAPIClientCodec;
        let mut buf = bytes::BytesMut::new();
        codec.encode(msg, &mut buf).unwrap();

        let data = buf.freeze();
        let data = std::str::from_utf8(&data).unwrap();

        // assert that we have the suffix \r\n
        assert!(data.ends_with("\r\n"));
        // assert that we have the prefix {"op":"authentication"
        assert!(data.starts_with("{\"op\":\"authentication\""));
    }
}

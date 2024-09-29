use alloc::sync::Arc;
use core::net::SocketAddr;
use core::pin::Pin;
use core::task::{Context, Poll};

use betfair_adapter::BetfairUrl;
use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::response::ResponseMessage;
use futures::Sink;
use futures_util::sink::SinkExt;
use futures_util::{Stream, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_util::bytes;
use tokio_util::codec::{Decoder, Encoder, Framed};

use crate::StreamError;

pub(crate) async fn connect(
    url: BetfairUrl<betfair_adapter::Stream>,
) -> Result<RawStreamApiConnection, StreamError> {
    let url = url.url();
    tracing::debug!(?url, "connecting to stream");

    let host = url.host_str().ok_or(StreamError::HostStringNotPresent)?;
    let port = url.port().unwrap_or(443);
    let socket_addr = tokio::net::lookup_host((host, port))
        .await
        .map_err(|err| {
            tracing::error!(?err, "unable to look up host");
            StreamError::UnableToLookUpHost {
                host: host.to_owned(),
                port,
            }
        })?
        .next();
    let domain = url.domain();
    match (domain, socket_addr) {
        (Some(domain), Some(socket_addr)) => {
            let connection = connect_tls(domain, socket_addr).await?;
            tracing::debug!("connecting to Stream API");
            Ok(connection)
        }
        #[cfg(feature = "integration-test")]
        (None, Some(socket_addr)) => {
            let connection = connect_tls("localhost", socket_addr).await?;
            tracing::debug!("connecting to Stream API");
            Ok(connection)
        }
        params => {
            tracing::error!(?params, "unable to connect to Stream API");

            Err(StreamError::MisconfiguredStreamURL)
        }
    }
}

#[tracing::instrument(err)]
async fn connect_tls(
    domain: &str,
    socket_addr: SocketAddr,
) -> Result<RawStreamApiConnection, StreamError> {
    let domain = rustls::pki_types::ServerName::try_from(domain.to_owned()).map_err(|err| {
        tracing::error!(?err, "unable to convert domain to server name");
        StreamError::UnableConvertDomainToServerName
    })?;
    let stream = TcpStream::connect(&socket_addr).await?;
    let connector = tls_connector()?;
    let stream = connector.connect(domain, stream).await.map_err(|err| {
        tracing::error!(?err, "unable to connect to TLS stream");
        StreamError::UnableConnectToTlsStream
    })?;
    let framed = Framed::new(stream, StreamAPIClientCodec);
    Ok(internal::RawStreamApiConnection { io: framed })
}

pub(crate) type RawStreamApiConnection =
    internal::RawStreamApiConnection<tokio_rustls::client::TlsStream<TcpStream>>;

mod internal {
    use super::*;

    pub(crate) struct RawStreamApiConnection<
        IO: AsyncRead + AsyncWrite + core::fmt::Debug + Send + Unpin,
    > {
        pub(super) io: Framed<IO, StreamAPIClientCodec>,
    }

    impl<IO: AsyncRead + AsyncWrite + core::fmt::Debug + Send + Unpin> Stream
        for RawStreamApiConnection<IO>
    {
        type Item = Result<ResponseMessage, CodecError>;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            self.io.poll_next_unpin(cx)
        }
    }

    impl<IO: AsyncRead + AsyncWrite + core::fmt::Debug + Send + Unpin> Sink<RequestMessage>
        for RawStreamApiConnection<IO>
    {
        type Error = CodecError;

        fn poll_ready(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            self.io.poll_ready_unpin(cx)
        }

        fn start_send(mut self: Pin<&mut Self>, item: RequestMessage) -> Result<(), Self::Error> {
            self.io.start_send_unpin(item)
        }

        fn poll_flush(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            self.io.poll_flush_unpin(cx)
        }

        fn poll_close(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            self.io.poll_close_unpin(cx)
        }
    }
}

pub(crate) struct StreamAPIClientCodec;

#[derive(Debug, thiserror::Error)]
pub(crate) enum CodecError {
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("IO Error {0}")]
    IoError(#[from] std::io::Error),
}

impl Decoder for StreamAPIClientCodec {
    type Item = ResponseMessage;
    type Error = CodecError;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        use itertools::Itertools;

        // Check if there is a newline character in the buffer
        if let Some(line) = src
            .iter()
            .tuple_windows::<(_, _)>()
            .position(|(char_a, char_b)| char_a == &b'\r' && char_b == &b'\n')
            .map(|idx| src.split_to(idx.saturating_add(2)))
        {
            if let Some((json, _delimiters)) = line.split_last_chunk::<2>() {
                // Deserialize the JSON data
                let data = serde_json::from_slice::<Self::Item>(json)?;
                return Ok(Some(data))
            }
        }
        Ok(None)
    }
}

impl Encoder<RequestMessage> for StreamAPIClientCodec {
    type Error = CodecError;

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

#[tracing::instrument(err)]
fn tls_connector() -> Result<tokio_rustls::TlsConnector, StreamError> {
    use tokio_rustls::TlsConnector;

    let mut roots = rustls::RootCertStore::empty();
    let native_certs = rustls_native_certs::load_native_certs().map_err(|err| {
        tracing::error!(?err, "Cannot load native certificates");
        StreamError::LocalCertificateLoadError
    })?;
    for cert in native_certs {
        roots.add(cert).map_err(|err| {
            tracing::error!(?err, "Cannot set native certificate");
            StreamError::CannotSetNativeCertificate
        })?;
    }

    #[cfg(feature = "integration-test")]
    {
        use crate::CERTIFICATE;

        let cert = rustls_pemfile::certs(
            &mut CERTIFICATE
                .get()
                .ok_or(StreamError::CustomCertificateNotSet)?
                .as_bytes(),
        )
        .next()
        .ok_or(StreamError::InvalidCustomCertificate)?
        .map_err(|_| StreamError::InvalidCustomCertificate)?;
        roots.add(cert).map_err(|err| {
            tracing::error!(?err, "Cannot set native certificate");
            StreamError::CustomCertificateNotSet
        })?;
    };

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(TlsConnector::from(Arc::new(config)))
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

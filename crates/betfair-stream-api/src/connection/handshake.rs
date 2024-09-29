use core::pin::Pin;
use core::task::Poll;

use betfair_adapter::{ApplicationKey, SessionToken};
use betfair_stream_types::request::{authentication_message, RequestMessage};
use betfair_stream_types::response::status_message::{StatusCode, StatusMessage};
use betfair_stream_types::response::ResponseMessage;
use futures::{SinkExt, Stream, StreamExt};

use super::cron::NeedsRestart;
use crate::tls_sream::RawStreamApiConnection;
use crate::{ExternalUpdates, MetadataUpdates};

#[pin_project::pin_project]
pub(crate) struct Handshake<'a> {
    session_token: &'a SessionToken,
    application_key: &'a ApplicationKey,
    #[pin]
    connection: Pin<&'a mut RawStreamApiConnection>,
    state: State,
}

impl<'a> Handshake<'a> {
    pub(crate) fn new(
        session_token: &'a SessionToken,
        application_key: &'a ApplicationKey,
        connection: Pin<&'a mut RawStreamApiConnection>,
    ) -> Self {
        Self {
            session_token,
            application_key,
            connection,
            state: State::AwaitConnectionMessage,
        }
    }
}

#[derive(Debug)]
enum State {
    Error,
    Done(StatusMessage),
    AwaitConnectionMessage,
    SendAuthenticationMessage,
    AwaitPollForConnectionMessage,
    AwaitStatusMessage,
}

impl<'a> Stream for Handshake<'a> {
    type Item = Result<ExternalUpdates<ResponseMessage>, NeedsRestart>;

    #[expect(clippy::too_many_lines)]
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.state {
            State::Error => Poll::Ready(None),
            State::Done(ref msg) => {
                let metadata_update = MetadataUpdates::Authenticated {
                    connections_available: msg.connections_available.unwrap_or(-1),
                    connection_id: msg.connection_id.clone(),
                };
                self.state = State::Error;
                cx.waker().wake_by_ref();
                Poll::Ready(Some(Ok(ExternalUpdates::Metadata(metadata_update))))
            }
            State::AwaitConnectionMessage => {
                let connection_message_poll = self.connection.poll_next_unpin(cx);
                match connection_message_poll {
                    Poll::Ready(None) => {
                        tracing::error!("Connection closed");
                        self.state = State::Error;
                        cx.waker().wake_by_ref();
                        Poll::Ready(Some(Err(NeedsRestart)))
                    }
                    Poll::Ready(Some(msg)) => match msg {
                        Ok(ResponseMessage::Connection(connection_message)) => {
                            self.state = State::SendAuthenticationMessage;
                            Poll::Ready(Some(Ok(ExternalUpdates::Layer(
                                ResponseMessage::Connection(connection_message),
                            ))))
                        }
                        Ok(msg) => {
                            self.state = State::Error;
                            cx.waker().wake_by_ref();
                            tracing::error!(msg =? msg, "Expected connection message, got something else");
                            Poll::Ready(Some(Err(NeedsRestart)))
                        }
                        Err(_) => {
                            self.state = State::Error;
                            cx.waker().wake_by_ref();
                            Poll::Ready(Some(Err(NeedsRestart)))
                        }
                    },
                    Poll::Pending => Poll::Pending,
                }
            }
            State::SendAuthenticationMessage => {
                let authorization_message = authentication_message::AuthenticationMessage {
                    id: Some(-1),
                    session: self.session_token.0.expose_secret().clone(),
                    app_key: self.application_key.0.expose_secret().clone(),
                };
                let send_result = self
                    .connection
                    .start_send_unpin(RequestMessage::Authentication(authorization_message));
                match send_result {
                    Ok(()) => {
                        self.state = State::AwaitPollForConnectionMessage;
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Err(err) => {
                        tracing::error!(error =? err, "Failed to send authentication message");
                        Poll::Ready(Some(Err(NeedsRestart)))
                    }
                }
            }
            State::AwaitPollForConnectionMessage => {
                return match self.connection.poll_flush_unpin(cx) {
                    Poll::Ready(Ok(())) => {
                        self.state = State::AwaitStatusMessage;
                        return Poll::Ready(Some(Ok(ExternalUpdates::Metadata(
                            MetadataUpdates::AuthenticationMessageSent,
                        ))));
                    }
                    Poll::Ready(Err(err)) => {
                        self.state = State::Error;
                        cx.waker().wake_by_ref();
                        tracing::error!(error =? err, "Failed to flush authentication message");
                        return Poll::Ready(Some(Err(NeedsRestart)));
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
            State::AwaitStatusMessage => {
                let status_message_poll = self.connection.poll_next_unpin(cx);
                match status_message_poll {
                    Poll::Ready(None) => {
                        tracing::error!("Connection closed");
                        Poll::Ready(Some(Err(NeedsRestart)))
                    }
                    Poll::Ready(Some(Ok(response_msg))) => match response_msg {
                        ResponseMessage::Status(ref status_message) => {
                            if status_message.status_code == Some(StatusCode::Success) {
                                self.state = State::Done(status_message.clone());
                                cx.waker().wake_by_ref();
                                Poll::Ready(Some(Ok(ExternalUpdates::Layer(response_msg))))
                            } else {
                                self.state = State::Error;
                                cx.waker().wake_by_ref();
                                tracing::error!(status_code =? status_message.status_code, "Failed to authenticate");
                                Poll::Ready(Some(Err(NeedsRestart)))
                            }
                        }
                        msg @ (ResponseMessage::Connection(_) |
                        ResponseMessage::MarketChange(_) |
                        ResponseMessage::OrderChange(_)) => {
                            self.state = State::Error;
                            cx.waker().wake_by_ref();
                            tracing::error!(msg =? msg, "Expected status message, got something else");
                            Poll::Ready(Some(Err(NeedsRestart)))
                        }
                    },
                    Poll::Ready(Some(Err(_))) => {
                        self.state = State::Error;
                        cx.waker().wake_by_ref();
                        tracing::error!("Failed to read status message");
                        Poll::Ready(Some(Err(NeedsRestart)))
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }
}

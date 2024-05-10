use std::pin::Pin;
use std::task::Poll;

use betfair_adapter::{ApplicationKey, SessionToken};
use betfair_stream_types::request::{authentication_message, RequestMessage};
use betfair_stream_types::response::status_message::{StatusCode, StatusMessage};
use betfair_stream_types::response::ResponseMessage;
use futures::{SinkExt, Stream, StreamExt};

use super::cron::NeedsRestart;
use crate::tls_sream::RawStreamApiConnection;
use crate::{ExternalUpdates, MetadataUpdates};

pub struct Handshake<'a> {
    session_token: &'a SessionToken,
    application_key: &'a ApplicationKey,
    connection: Pin<&'a mut RawStreamApiConnection>,
    state: State,
}

impl<'a> Handshake<'a> {
    pub fn new(
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
impl<'a> Unpin for Handshake<'a> {}

impl<'a> Stream for Handshake<'a> {
    type Item = Result<ExternalUpdates<ResponseMessage>, NeedsRestart>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match &mut self.state {
            State::Error => Poll::Ready(None),
            State::Done(msg) => {
                let metadata_update = MetadataUpdates::Authenticated {
                    connections_available: msg.connections_available.unwrap_or(-1),
                    connection_id: msg.connection_id.clone(),
                };
                self.state = State::Error;
                cx.waker().wake_by_ref();
                Poll::Ready(Some(Ok(ExternalUpdates::Metadata(metadata_update))))
            }
            State::AwaitConnectionMessage => {
                let connection_message = self.connection.poll_next_unpin(cx);
                match connection_message {
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
                    Ok(_) => {
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
                    Poll::Ready(Ok(_)) => {
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
                let status_message = self.connection.poll_next_unpin(cx);
                match status_message {
                    Poll::Ready(None) => {
                        tracing::error!("Connection closed");
                        Poll::Ready(Some(Err(NeedsRestart)))
                    }
                    Poll::Ready(Some(Ok(msg))) => match &msg {
                        ResponseMessage::Status(status_message) => {
                            if status_message.status_code == Some(StatusCode::Success) {
                                self.state = State::Done(status_message.clone());
                                cx.waker().wake_by_ref();
                                Poll::Ready(Some(Ok(ExternalUpdates::Layer(msg))))
                            } else {
                                self.state = State::Error;
                                cx.waker().wake_by_ref();
                                tracing::error!(status_code =? status_message.status_code, "Failed to authenticate");
                                Poll::Ready(Some(Err(NeedsRestart)))
                            }
                        }
                        msg => {
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

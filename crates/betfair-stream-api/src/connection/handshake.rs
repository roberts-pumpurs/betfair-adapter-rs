use std::convert::Infallible as Never;
use std::pin::Pin;
use std::task::Poll;
use std::time::Duration;

use betfair_adapter::{ApplicationKey, BetfairUrl, SessionToken};
use betfair_stream_types::request::{authentication_message, RequestMessage};
use betfair_stream_types::response::connection_message::ConnectionMessage;
use betfair_stream_types::response::market_change_message::MarketChangeMessage;
use betfair_stream_types::response::order_change_message::OrderChangeMessage;
use betfair_stream_types::response::status_message::{StatusCode, StatusMessage};
use betfair_stream_types::response::ResponseMessage;
use futures::task::SpawnExt;
use futures::{pin_mut, FutureExt, SinkExt, Stream, TryFutureExt, TryStreamExt};
use futures_concurrency::prelude::*;
use tokio::runtime::Handle;
use tokio::task::JoinSet;
use tokio_stream::StreamExt;

use crate::connection::cron::AsyncTaskStopReason;
use crate::tls_sream::RawStreamApiConnection;
use crate::{ExternalUpdates, StreamError};

pub async fn handshake<'a>(
    session_token: SessionToken,
    application_key: ApplicationKey,
    connection: &mut Pin<&'a mut RawStreamApiConnection>,
    sender: tokio::sync::mpsc::Sender<ExternalUpdates<ResponseMessage>>,
) -> Result<StatusMessage, AsyncTaskStopReason> {
    async fn read_next<'a, S: Stream<Item = Result<ResponseMessage, StreamError>>>(
        read: &mut Pin<&'a mut S>,
    ) -> Result<ResponseMessage, AsyncTaskStopReason> {
        read.next()
            .await
            .ok_or(AsyncTaskStopReason::NeedsRestart)?
            .map_err(|_| AsyncTaskStopReason::NeedsRestart)
    }

    // get connection message
    let connection_message = read_next(connection).await?;
    let ResponseMessage::Connection(connection_message) = connection_message else {
        tracing::error!(
            msg =? connection_message,
            "Expected connection message, got something else"
        );
        return Err(AsyncTaskStopReason::NeedsRestart);
    };
    sender
        .send(ExternalUpdates::Layer(ResponseMessage::Connection(
            connection_message.clone(),
        )))
        .await
        .map_err(|_| AsyncTaskStopReason::FatalError)?;

    // send authentication message
    let authorization_message = authentication_message::AuthenticationMessage {
        id: Some(-1),
        session: session_token.0.expose_secret().clone(),
        app_key: application_key.0.expose_secret().clone(),
    };
    connection
        .send(RequestMessage::Authentication(authorization_message))
        .await
        .map_err(|_| AsyncTaskStopReason::FatalError)?;

    // get status message
    let status_message = read_next(connection).await?;
    let ResponseMessage::Status(status_message) = status_message else {
        tracing::error!(
            msg =? status_message,
            "Expected status message, got something else"
        );
        return Err(AsyncTaskStopReason::NeedsRestart);
    };
    sender
        .send(ExternalUpdates::Layer(ResponseMessage::Status(
            status_message.clone(),
        )))
        .await
        .map_err(|_| AsyncTaskStopReason::FatalError)?;

    // parse status message
    if status_message
        .status_code
        .map(|x| x == StatusCode::Success)
        .unwrap_or(false)
    {
        tracing::info!("Successfully authenticated");
    } else {
        tracing::error!(
            msg =? status_message,
            "Failed to authenticate, got status message"
        );
        return Err(AsyncTaskStopReason::NeedsRestart);
    }
    Ok(status_message)
}

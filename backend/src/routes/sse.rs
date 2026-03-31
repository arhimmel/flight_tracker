use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use futures::stream::{self, StreamExt};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use crate::models::AlertEvent;

pub async fn sse_handler(
    State(tx): State<broadcast::Sender<AlertEvent>>,
) -> impl IntoResponse {
    let rx = tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| async move {
        match msg {
            Ok(event) => {
                let data = serde_json::to_string(&event).ok()?;
                Some(Ok::<Event, anyhow::Error>(
                    Event::default().event("price_drop").data(data),
                ))
            }
            Err(_) => None,
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

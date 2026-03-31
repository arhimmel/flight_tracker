use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use futures::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use crate::AppState;

pub async fn sse_handler(State(state): State<AppState>) -> impl IntoResponse {
    let rx = state.tx.subscribe();
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

use std::sync::Arc;

use axum::{
    routing::{delete, get, post},
    Router,
};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod db;
mod models;
mod poller;
mod routes;

use models::AlertEvent;
use poller::{
    price_fetcher::PriceFetcher,
    providers::{kiwi::KiwiProvider, mock::MockProvider},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env if present
    dotenvy::dotenv().ok();

    // Tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://data/tracker.db".to_string());

    // Ensure data directory exists
    if let Some(parent) = std::path::Path::new(&database_url.replace("sqlite://", "")).parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }

    let pool = db::init_pool(&database_url).await?;
    tracing::info!(database_url, "Database connected");

    // Price fetcher — swap provider via PRICE_PROVIDER env var
    let fetcher: Arc<dyn PriceFetcher> = match std::env::var("PRICE_PROVIDER")
        .unwrap_or_else(|_| "mock".to_string())
        .as_str()
    {
        "kiwi" => {
            let api_key = std::env::var("KIWI_API_KEY")
                .expect("KIWI_API_KEY must be set when PRICE_PROVIDER=kiwi");
            Arc::new(KiwiProvider::new(api_key))
        }
        _ => {
            tracing::warn!("Using MockProvider — set PRICE_PROVIDER=kiwi for real prices");
            Arc::new(MockProvider::default())
        }
    };

    // SSE broadcast channel
    let (tx, _rx) = broadcast::channel::<AlertEvent>(64);

    // Background polling loop
    tokio::spawn(poller::start_polling_loop(
        pool.clone(),
        Arc::clone(&fetcher),
        tx.clone(),
    ));

    // CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Router
    let app = Router::new()
        .route("/alerts", post(routes::alerts::create_alert))
        .route("/alerts", get(routes::alerts::list_alerts))
        .route("/alerts/:id", delete(routes::alerts::delete_alert))
        .route("/events", get(routes::sse::sse_handler))
        .with_state(pool)
        // SSE route needs the broadcast sender as state — override with a nested router
        .layer(cors);

    // We need two different state types; use a nested router for SSE
    let app = Router::new()
        .route("/events", get(routes::sse::sse_handler).with_state(tx))
        .route("/alerts", post(routes::alerts::create_alert).with_state(pool.clone()))
        .route("/alerts", get(routes::alerts::list_alerts).with_state(pool.clone()))
        .route("/alerts/:id", delete(routes::alerts::delete_alert).with_state(pool.clone()))
        .layer(cors);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(addr, "Server listening");

    axum::serve(listener, app).await?;
    Ok(())
}

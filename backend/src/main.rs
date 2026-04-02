use std::sync::Arc;

use axum::{
    routing::{delete, get, post},
    Router,
};
use sqlx::SqlitePool;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod auth;
mod db;
mod email;
mod models;
mod poller;
mod routes;

use models::AlertEvent;
use poller::{
    price_fetcher::PriceFetcher,
    providers::{kiwi::KiwiProvider, mock::MockProvider},
};

/// Shared application state passed to every route handler.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub tx: broadcast::Sender<AlertEvent>,
}

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

    let state = AppState { pool, tx };

    // CORS — allow Authorization header so the frontend can send Bearer tokens
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Auth (unauthenticated)
        .route("/auth/request-otp", post(routes::auth::request_otp))
        .route("/auth/verify-otp", post(routes::auth::verify_otp))
        // Alerts (require Bearer token)
        .route("/alerts", post(routes::alerts::create_alert))
        .route("/alerts", get(routes::alerts::list_alerts))
        .route("/alerts/:id", delete(routes::alerts::delete_alert))
        // SSE
        .route("/events", get(routes::sse::sse_handler))
        .layer(cors)
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(addr, "Server listening");

    axum::serve(listener, app).await?;
    Ok(())
}

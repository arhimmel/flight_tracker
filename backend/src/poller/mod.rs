pub mod price_fetcher;
pub mod providers;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::Utc;
use futures::future::join_all;
use sqlx::SqlitePool;
use tokio::sync::broadcast;

use crate::models::{Alert, AlertEvent};
use price_fetcher::PriceFetcher;

pub async fn start_polling_loop(
    pool: SqlitePool,
    fetcher: Arc<dyn PriceFetcher>,
    tx: broadcast::Sender<AlertEvent>,
) {
    let interval_mins: u64 = std::env::var("POLL_INTERVAL_MINS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);

    tracing::info!(interval_mins, "Price polling loop started");

    let mut interval = tokio::time::interval(Duration::from_secs(interval_mins * 60));

    loop {
        interval.tick().await;
        if let Err(e) = check_all_active_alerts(&pool, &fetcher, &tx).await {
            tracing::error!("Polling cycle error: {e}");
        }
    }
}

async fn check_all_active_alerts(
    pool: &SqlitePool,
    fetcher: &Arc<dyn PriceFetcher>,
    tx: &broadcast::Sender<AlertEvent>,
) -> Result<()> {
    let alerts = sqlx::query_as::<_, Alert>(
        "SELECT * FROM alerts WHERE status = 'active'"
    )
    .fetch_all(pool)
    .await?;

    tracing::info!(count = alerts.len(), "Checking active alerts");

    let tasks: Vec<_> = alerts
        .into_iter()
        .map(|alert| {
            let pool = pool.clone();
            let fetcher = Arc::clone(fetcher);
            let tx = tx.clone();
            tokio::spawn(async move {
                if let Err(e) = check_single_alert(&pool, &fetcher, &tx, alert).await {
                    tracing::warn!("Alert check failed: {e}");
                }
            })
        })
        .collect();

    join_all(tasks).await;
    Ok(())
}

async fn check_single_alert(
    pool: &SqlitePool,
    fetcher: &Arc<dyn PriceFetcher>,
    tx: &broadcast::Sender<AlertEvent>,
    alert: Alert,
) -> Result<()> {
    let result = fetcher
        .fetch_price(&alert.flight_number, &alert.origin, &alert.destination, &alert.flight_date)
        .await?;

    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO price_history (alert_id, price, checked_at) VALUES (?, ?, ?)"
    )
    .bind(alert.id)
    .bind(result.price)
    .bind(&now)
    .execute(pool)
    .await?;

    let triggered = result.price <= alert.target_price;
    let new_status = if triggered { "triggered" } else { "active" };
    let notified_at: Option<&str> = if triggered { Some(&now) } else { None };

    sqlx::query(
        "UPDATE alerts SET current_price = ?, last_checked = ?, status = ?, notified_at = COALESCE(notified_at, ?) WHERE id = ?"
    )
    .bind(result.price)
    .bind(&now)
    .bind(new_status)
    .bind(notified_at)
    .bind(alert.id)
    .execute(pool)
    .await?;

    tracing::debug!(
        alert_id = alert.id,
        flight = alert.flight_number,
        price = result.price,
        target = alert.target_price,
        triggered,
        provider = result.provider,
        "Alert checked"
    );

    if triggered {
        let event = AlertEvent {
            alert_id: alert.id,
            flight_number: alert.flight_number,
            flight_date: alert.flight_date,
            origin: alert.origin,
            destination: alert.destination,
            target_price: alert.target_price,
            current_price: result.price,
        };
        let _ = tx.send(event);
    }

    Ok(())
}

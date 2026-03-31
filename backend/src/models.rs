use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Alert {
    pub id: i64,
    pub flight_number: String,
    pub flight_date: String,
    pub origin: String,
    pub destination: String,
    pub target_price: f64,
    pub current_price: Option<f64>,
    pub status: String,
    pub created_at: String,
    pub last_checked: Option<String>,
    pub notified_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAlertRequest {
    pub flight_number: String,
    pub flight_date: String,
    pub origin: String,
    pub destination: String,
    pub target_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub alert_id: i64,
    pub flight_number: String,
    pub flight_date: String,
    pub origin: String,
    pub destination: String,
    pub target_price: f64,
    pub current_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PriceHistory {
    pub id: i64,
    pub alert_id: i64,
    pub price: f64,
    pub checked_at: String,
}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sqlx::SqlitePool;

use crate::models::{Alert, CreateAlertRequest};

pub async fn create_alert(
    State(pool): State<SqlitePool>,
    Json(req): Json<CreateAlertRequest>,
) -> Result<(StatusCode, Json<Alert>), (StatusCode, String)> {
    let now = Utc::now().to_rfc3339();

    let id = sqlx::query!(
        r#"
        INSERT INTO alerts (flight_number, flight_date, origin, destination, target_price, status, created_at)
        VALUES (?, ?, ?, ?, ?, 'active', ?)
        "#,
        req.flight_number,
        req.flight_date,
        req.origin,
        req.destination,
        req.target_price,
        now,
    )
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .last_insert_rowid();

    let alert = sqlx::query_as!(Alert, "SELECT * FROM alerts WHERE id = ?", id)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(alert)))
}

pub async fn list_alerts(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Alert>>, (StatusCode, String)> {
    let alerts = sqlx::query_as!(Alert, "SELECT * FROM alerts ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(alerts))
}

pub async fn delete_alert(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query!("DELETE FROM alerts WHERE id = ?", id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, format!("Alert {id} not found")));
    }

    Ok(StatusCode::NO_CONTENT)
}

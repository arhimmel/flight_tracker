use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sqlx::Row;

use crate::{models::{Alert, CreateAlertRequest}, AppState};

pub async fn create_alert(
    State(state): State<AppState>,
    Json(req): Json<CreateAlertRequest>,
) -> Result<(StatusCode, Json<Alert>), (StatusCode, String)> {
    let now = Utc::now().to_rfc3339();

    let id: i64 = sqlx::query(
        "INSERT INTO alerts (flight_number, flight_date, origin, destination, target_price, status, created_at)
         VALUES (?, ?, ?, ?, ?, 'active', ?)"
    )
    .bind(&req.flight_number)
    .bind(&req.flight_date)
    .bind(&req.origin)
    .bind(&req.destination)
    .bind(req.target_price)
    .bind(&now)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .last_insert_rowid();

    let alert = fetch_alert_by_id(&state, id).await?;
    Ok((StatusCode::CREATED, Json(alert)))
}

pub async fn list_alerts(
    State(state): State<AppState>,
) -> Result<Json<Vec<Alert>>, (StatusCode, String)> {
    let alerts = sqlx::query_as::<_, Alert>(
        "SELECT * FROM alerts ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(alerts))
}

pub async fn delete_alert(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM alerts WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, format!("Alert {id} not found")));
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn fetch_alert_by_id(state: &AppState, id: i64) -> Result<Alert, (StatusCode, String)> {
    sqlx::query_as::<_, Alert>("SELECT * FROM alerts WHERE id = ?")
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

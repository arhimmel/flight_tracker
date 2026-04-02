use axum::{extract::State, http::StatusCode, Json};
use chrono::{Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::Row;
use uuid::Uuid;

use crate::{email, AppState};

#[derive(Deserialize)]
pub struct RequestOtpPayload {
    pub email: String,
}

#[derive(Deserialize)]
pub struct VerifyOtpPayload {
    pub email: String,
    pub otp: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub email: String,
}

fn hash_otp(otp: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(otp.as_bytes());
    hex::encode(hasher.finalize())
}

/// POST /auth/request-otp
/// Generates a 6-digit OTP, stores its hash, and emails it to the user.
/// Always returns 204 to avoid leaking whether an address is registered.
pub async fn request_otp(
    State(state): State<AppState>,
    Json(payload): Json<RequestOtpPayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    let email_addr = payload.email.trim().to_lowercase();

    let otp: String = {
        let mut rng = rand::thread_rng();
        format!("{:06}", rng.gen_range(0..1_000_000u32))
    };

    let token_hash = hash_otp(&otp);
    let expires_at = (Utc::now() + Duration::minutes(15)).to_rfc3339();

    // Replace any unused pending codes for this address
    sqlx::query("DELETE FROM otp_tokens WHERE email = ? AND used = 0")
        .bind(&email_addr)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query(
        "INSERT INTO otp_tokens (email, token_hash, expires_at, used) VALUES (?, ?, ?, 0)",
    )
    .bind(&email_addr)
    .bind(&token_hash)
    .bind(&expires_at)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    email::send_otp_email(&email_addr, &otp)
        .await
        .map_err(|e| {
            tracing::error!("Failed to send OTP email to {}: {}", email_addr, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to send email".to_string(),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /auth/verify-otp
/// Verifies the code, creates the user if new, issues a 30-day session token.
pub async fn verify_otp(
    State(state): State<AppState>,
    Json(payload): Json<VerifyOtpPayload>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let email_addr = payload.email.trim().to_lowercase();
    let token_hash = hash_otp(payload.otp.trim());
    let now = Utc::now().to_rfc3339();

    let row = sqlx::query(
        "SELECT id FROM otp_tokens \
         WHERE email = ? AND token_hash = ? AND used = 0 AND expires_at > ? \
         ORDER BY id DESC LIMIT 1",
    )
    .bind(&email_addr)
    .bind(&token_hash)
    .bind(&now)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let otp_id: i64 = match row {
        Some(r) => r.get("id"),
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Invalid or expired code".to_string(),
            ))
        }
    };

    // Consume the OTP immediately so it can't be reused
    sqlx::query("UPDATE otp_tokens SET used = 1 WHERE id = ?")
        .bind(otp_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Create user on first login; do nothing if the email already exists
    let new_user_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO users (id, email, created_at) VALUES (?, ?, ?) ON CONFLICT(email) DO NOTHING",
    )
    .bind(&new_user_id)
    .bind(&email_addr)
    .bind(&created_at)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user_id: String = sqlx::query_scalar("SELECT id FROM users WHERE email = ?")
        .bind(&email_addr)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let session_id = Uuid::new_v4().to_string();
    let session_expires = (Utc::now() + Duration::days(30)).to_rfc3339();
    let session_created = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO sessions (id, user_id, expires_at, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(&session_id)
    .bind(&user_id)
    .bind(&session_expires)
    .bind(&session_created)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AuthResponse {
        token: session_id,
        email: email_addr,
    }))
}

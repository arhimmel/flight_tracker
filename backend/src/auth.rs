use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use chrono::Utc;
use sqlx::Row;

use crate::AppState;

/// Injected by the `FromRequestParts` impl into any handler that declares it
/// as a parameter. Returns 401 if the `Authorization: Bearer <token>` header
/// is missing, invalid, or expired.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub email: String,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(|t| t.trim().to_string())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Missing Authorization header".to_string(),
                )
            })?;

        let now = Utc::now().to_rfc3339();

        let row = sqlx::query(
            "SELECT s.user_id, u.email \
             FROM sessions s \
             JOIN users u ON u.id = s.user_id \
             WHERE s.id = ? AND s.expires_at > ?",
        )
        .bind(&token)
        .bind(&now)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        match row {
            Some(r) => Ok(AuthUser {
                user_id: r.get("user_id"),
                email: r.get("email"),
            }),
            None => Err((
                StatusCode::UNAUTHORIZED,
                "Invalid or expired session".to_string(),
            )),
        }
    }
}

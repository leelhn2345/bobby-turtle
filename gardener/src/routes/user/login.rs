use std::time::Instant;

use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::ToSchema;
use validator::Validate;

use crate::auth::AuthSession;

use super::{analyze_password, UserError};

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LoginCredentials {
    #[validate(email)]
    #[schema(default = "user@email.com")]
    pub username: String,

    #[validate(custom(function = "analyze_password"))]
    #[schema(default = "1Q2w3e4r5t!~")]
    pub password: String,
}

/// user login
#[utoipa::path(
    post,
    tag="user",
    path="/login",
    responses(
        (status = StatusCode::OK, description = "user successfully logged in"),
        (status = StatusCode::UNPROCESSABLE_ENTITY, description = "validation error"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "internal server error")
    )
)]
#[tracing::instrument(skip_all,fields(username = login_creds.username))]
pub async fn login(
    mut auth_session: AuthSession,
    State(pool): State<PgPool>,
    Json(login_creds): Json<LoginCredentials>,
) -> Result<StatusCode, UserError> {
    tracing::debug!("wtf");
    let now = Instant::now();
    login_creds.validate()?;
    let user_id = validate_credentials(login_creds, &pool).await?;
    let elapsed = now.elapsed().as_secs_f32();
    tracing::debug!("{elapsed}s has elapsed");
    Err(UserError::InvalidCredentials)
    // Ok(StatusCode::OK)
}

#[tracing::instrument(skip_all)]
async fn validate_credentials(
    creds: LoginCredentials,
    pool: &PgPool,
) -> Result<uuid::Uuid, UserError> {
    let user_id: Option<uuid::Uuid> = None;

    let valid_user_id = user_id.ok_or(UserError::InvalidCredentials)?;
    Ok(valid_user_id)
}

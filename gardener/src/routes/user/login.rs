use std::time::Instant;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::ToSchema;

use crate::auth::AuthSession;

use super::{analyze_password, UserError};

#[derive(Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoginCredentials {
    #[schema(default = "user@email.com")]
    pub username: String,

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
    Json(login_creds): Json<LoginCredentials>,
) -> Result<StatusCode, UserError> {
    let now = Instant::now();

    // let user = match auth_session.
    //
    let user = match auth_session.authenticate(login_creds).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(UserError::InvalidCredentials);
        }
        Err(_) => {
            return Err(UserError::InvalidCredentials);
        }
    };

    if auth_session.login(&user).await.is_err() {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let elapsed = now.elapsed().as_secs_f32();
    tracing::debug!("{elapsed}s has elapsed");
    // Err(UserError::InvalidCredentials)
    Ok(StatusCode::OK)
}

/// user logout
#[utoipa::path(
    get,
    tag="user",
    path="/logout",
    responses(
        (status = StatusCode::OK, description = "user successfully logged out"),
    )
)]
pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
    match auth_session.logout().await {
        Ok(_) => "logged out".into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

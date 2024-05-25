use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::task;
use utoipa::ToSchema;
use validator::{Validate, ValidationErrors};

use crate::auth::{AuthSession, Backend, PermissionLevel};

pub mod change_password;
pub mod sign_up;

#[derive(Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[validate(email)]
    #[schema(default = "user@email.com")]
    username: String,

    #[schema(default = "alpha")]
    first_name: String,
    #[schema(default = "user")]
    last_name: Option<String>,

    #[serde(skip_deserializing, default = "PermissionLevel::member")]
    permission_level: PermissionLevel,
}

#[derive(Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoginCredentials {
    #[schema(default = "user@email.com")]
    pub username: String,

    #[schema(default = "1Q2w3e4r5t!~")]
    pub password: String,
}

#[derive(thiserror::Error, Debug)]
pub enum UserError {
    #[error("username is taken")]
    UsernameTaken,

    #[error(transparent)]
    Validation(#[from] ValidationErrors),

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("new password is same as old password")]
    SamePassword,

    #[error("unknown error")]
    UnknownError(#[from] anyhow::Error),

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Authentication(#[from] axum_login::Error<Backend>),

    #[error(transparent)]
    TaskJoin(#[from] task::JoinError),
}

impl IntoResponse for UserError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (status_code, msg) = match self {
            Self::UsernameTaken => (StatusCode::CONFLICT, "username is taken".to_owned()),
            Self::Validation(e) => {
                tracing::error!("{e:#?}");
                let fields: Vec<&str> = e.field_errors().into_keys().collect();
                let field_string = fields.join(", ");
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!("validation error in {field_string}"),
                )
            }
            Self::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "invalid credentials".to_owned())
            }

            Self::SamePassword => (
                StatusCode::UNAUTHORIZED,
                "new password is same as old password".to_owned(),
            ),

            Self::UnknownError(e) => {
                tracing::error!("{e:#?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_owned(),
                )
            }

            Self::Database(e) => {
                tracing::error!("{e:#?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_owned(),
                )
            }

            Self::Authentication(e) => {
                tracing::error!("{e:#?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_owned(),
                )
            }
            Self::TaskJoin(e) => {
                tracing::error!("{e:#?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_owned(),
                )
            }
        };

        (status_code, Json(ErrorResponse { message: msg })).into_response()
    }
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
pub async fn login(
    mut auth_session: AuthSession,
    Json(login_creds): Json<LoginCredentials>,
) -> Result<StatusCode, UserError> {
    let user = match auth_session.authenticate(login_creds).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(UserError::InvalidCredentials);
        }
        Err(e) => return Err(UserError::UnknownError(e.into())),
    };

    if auth_session.login(&user).await.is_err() {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

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
pub async fn logout(mut auth_session: AuthSession) -> Result<Json<Value>, UserError> {
    match auth_session.logout().await {
        Ok(_) => Ok(Json(json!({"message":"user logged out"}))),
        Err(e) => Err(UserError::UnknownError(e.into())),
    }
}

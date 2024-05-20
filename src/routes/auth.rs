use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;
use validator::{Validate, ValidationErrors};

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("username is taken")]
    UsernameTaken,

    #[error(transparent)]
    ValidationError(#[from] ValidationErrors),

    #[error("invalid credentials")]
    InvalidCredentials,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (status_code, msg) = match self {
            Self::UsernameTaken => (StatusCode::CONFLICT, "username is taken".to_owned()),
            Self::ValidationError(_) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation error".to_owned(),
            ),
            Self::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "invalid credentials".to_owned())
            }
        };

        (status_code, Json(ErrorResponse { message: msg })).into_response()
    }
}

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LoginCredentials {
    #[validate(email)]
    username: String,
    #[validate(length(min = 8, max = 20))]
    password: String,
}

/// user login
#[utoipa::path(
    post,
    tag="auth",
    path="/login",
    responses(
        (status = StatusCode::OK, description = "user successfully logged in"),
        (status = StatusCode::UNPROCESSABLE_ENTITY, description = "validation error")
    )
)]
#[tracing::instrument(skip_all,fields(username = login_creds.username))]
pub async fn login(
    State(pool): State<PgPool>,
    Json(login_creds): Json<LoginCredentials>,
) -> Result<StatusCode, AuthError> {
    login_creds.validate()?;
    let user_id = validate_credentials(login_creds, &pool).await?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewUserCredentials {
    #[serde(flatten)]
    account_details: LoginCredentials,
    first_name: String,
    last_name: String,
}

/// new user
#[utoipa::path(
    post,
    tag="auth",
    path="/register",
    responses(
        (status = StatusCode::OK, description = "user successfully registered"),
        (status = StatusCode::CONFLICT, description = "username is taken"),
        (status = StatusCode::UNPROCESSABLE_ENTITY, description = "validation error")
    )
)]
#[tracing::instrument(skip_all,fields(username = new_user_creds.account_details.username))]
pub async fn register_new_user(
    State(pool): State<PgPool>,
    Json(new_user_creds): Json<NewUserCredentials>,
) -> Result<StatusCode, AuthError> {
    new_user_creds.account_details.validate()?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(skip_all)]
async fn validate_credentials(
    creds: LoginCredentials,
    pool: &PgPool,
) -> Result<uuid::Uuid, AuthError> {
    let user_id: Option<uuid::Uuid> = None;

    let valid_user_id = user_id.ok_or(AuthError::InvalidCredentials)?;
    Ok(valid_user_id)
}

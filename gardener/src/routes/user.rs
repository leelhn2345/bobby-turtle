use crate::auth::{AuthSession, Backend, PermissionLevel};
use anyhow::anyhow;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post, put},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use axum_login::login_required;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::task;
use tower_sessions::cookie::{time::Duration, Cookie, SameSite};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use super::AppState;

pub mod change_password;
pub mod sign_up;

#[derive(Deserialize, Serialize, sqlx::FromRow, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_field_names)]
pub struct User {
    #[serde(skip, default = "Uuid::new_v4")]
    user_id: Uuid,

    #[validate(email)]
    #[schema(default = "user@email.com")]
    username: String,

    #[schema(default = "alpha")]
    first_name: String,

    #[schema(default = "user")]
    last_name: Option<String>,

    #[serde(skip_deserializing, default = "Utc::now")]
    joined_at: DateTime<Utc>,

    #[serde(skip_deserializing, default = "Utc::now")]
    last_updated: DateTime<Utc>,

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

impl LoginCredentials {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
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
                StatusCode::BAD_REQUEST,
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
    path="/user/login",
    responses(
        (status = StatusCode::OK, description = "user successfully logged in"),
        (status = StatusCode::UNPROCESSABLE_ENTITY, description = "validation error"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "internal server error")
    )
)]
pub async fn login(
    mut auth_session: AuthSession,
    Json(login_creds): Json<LoginCredentials>,
) -> Result<Redirect, UserError> {
    let user = match auth_session.authenticate(login_creds).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(UserError::InvalidCredentials);
        }
        Err(e) => return Err(UserError::UnknownError(e.into())),
    };

    if auth_session.login(&user).await.is_err() {
        return Err(UserError::UnknownError(anyhow!("can't log in")));
    }

    Ok(Redirect::to("/user/user-info"))
}

/// user logout
#[utoipa::path(
    get,
    tag="user",
    path="/user/logout",
    responses(
        (status = StatusCode::OK, description = "user successfully logged out"),
    )
)]
pub async fn logout(
    mut auth_session: AuthSession,
    State(app): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<Value>), UserError> {
    match auth_session.logout().await {
        Ok(_) => Ok((
            jar.remove(Cookie::build("userInfo").domain(app.domain)),
            Json(json!({"message":"user logged out"})),
        )),
        Err(e) => Err(UserError::UnknownError(e.into())),
    }
}

/// user info
///
/// more fields will be shown than what is shown in the schema.
#[utoipa::path(
    get,
    tag="user",
    path="/user/user-info",
    responses(
        (status = StatusCode::OK, body = User, description = "user data"),
    )
)]
pub async fn user_info(
    jar: CookieJar,
    auth_session: AuthSession,
    State(app): State<AppState>,
) -> Result<(CookieJar, Json<User>), UserError> {
    let Some(verified_user) = auth_session.user else {
        return Err(anyhow!("user is not registered in auth session").into());
    };

    let pool = app.pool;

    let user: User = sqlx::query_as("select * from users where user_id = $1")
        .bind(verified_user.user_id)
        .fetch_one(&pool)
        .await?;

    let permission_cookie = Cookie::build((
        "userInfo",
        json!({
            "firstName":user.first_name,
            "lastName":user.last_name,
            "permission":user.permission_level
        })
        .to_string(),
    ))
    .http_only(true)
    .same_site(SameSite::None)
    .max_age(Duration::weeks(2))
    .domain(app.domain)
    .path("/")
    .secure(true);

    Ok((jar.add(permission_cookie), Json(user)))
}

pub fn user_router() -> Router<AppState> {
    Router::new()
        .route("/logout", get(logout))
        .route("/change-password", put(change_password::change_password))
        .route("/user-info", get(user_info))
        .route_layer(login_required!(Backend))
        .route("/sign-up", post(sign_up::register_new_user))
        .route("/login", post(login))
}

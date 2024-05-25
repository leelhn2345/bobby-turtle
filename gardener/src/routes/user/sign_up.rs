use axum::{extract::State, http::StatusCode, Json};
use password_auth::generate_hash;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{types::Json, PgPool};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::auth::{AuthSession, AuthenticatedUser};

use super::{analyze_password, User, UserError};

#[derive(Deserialize, Validate, ToSchema)]
pub struct NewUser {
    #[serde(flatten)]
    user_info: User,

    #[validate(custom(function = "analyze_password"))]
    #[schema(default = "1Q2w3e4r5t!~")]
    password: String,
}

/// new user
#[utoipa::path(
    post,
    tag="user",
    path="/sign_up",
    responses(
        (status = StatusCode::OK, description = "user successfully registered"),
        (status = StatusCode::CONFLICT, description = "username is taken"),
        (status = StatusCode::UNPROCESSABLE_ENTITY, description = "validation error"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "internal server error")
    )
)]
#[tracing::instrument(skip_all, fields(username = new_user.user_info.username))]
pub async fn register_new_user(
    mut auth_session: AuthSession,
    State(pool): State<PgPool>,
    Json(new_user): Json<NewUser>,
) -> Result<Json<Value>, UserError> {
    new_user.validate()?;

    let user_exists = sqlx::query!(
        "select from users where username = $1",
        new_user.user_info.username
    )
    .fetch_optional(&pool)
    .await?;

    if user_exists.is_some() {
        return Err(UserError::UsernameTaken);
    };

    let uuid_v4 = Uuid::new_v4();
    let password_hash = generate_hash(new_user.password);

    let user = new_user.user_info;

    sqlx::query!(
        "insert into users (user_id,username,password_hash)
        values ($1,$2,$3)",
        uuid_v4,
        user.username,
        password_hash
    )
    .execute(&pool)
    .await?;

    let auth_user = AuthenticatedUser::new(uuid_v4, password_hash);

    auth_session.login(&auth_user).await?;

    Ok(Json(json!({"message":"user successfully created"})))
}

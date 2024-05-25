use axum::{extract::State, Json};
use password_auth::generate_hash;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio::task;
use utoipa::ToSchema;
use validator::Validate;

use crate::auth::AuthSession;

use super::{sign_up::analyze_password, LoginCredentials, UserError};

#[derive(Validate, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewPassword {
    #[serde(flatten)]
    old_info: LoginCredentials,
    #[validate(custom(function = "analyze_password"))]
    #[schema(default = "1Q2w3e4r5t6Y!~")]
    new_password: String,
}

#[utoipa::path(
    post,
    tag="user",
    path="/change-password",
    responses(
        (status = StatusCode::OK, description = "password changed")
    )
)]
pub async fn change_password(
    auth_session: AuthSession,
    State(pool): State<PgPool>,
    Json(new_password): Json<NewPassword>,
) -> Result<Json<Value>, UserError> {
    if new_password.old_info.password == new_password.new_password {
        return Err(UserError::SamePassword);
    };
    let user_exists = auth_session
        .authenticate(new_password.old_info.clone())
        .await?;

    if user_exists.is_none() {
        return Err(UserError::InvalidCredentials);
    }

    new_password.validate()?;

    let username = new_password.old_info.username;

    let password_hash = task::spawn_blocking(|| generate_hash(new_password.new_password)).await?;

    sqlx::query!(
        "update users 
        set password_hash = $1 
        where username = $2",
        password_hash,
        username
    )
    .execute(&pool)
    .await?;

    Ok(Json(json!({"message":"password successfully changed"})))
}

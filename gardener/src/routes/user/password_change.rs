use anyhow::Context;
use axum::{extract::State, Json};
use password_auth::{generate_hash, verify_password};
use serde::Deserialize;
use serde_json::{json, Value};
use time::OffsetDateTime;
use tokio::task;
use utoipa::ToSchema;
use validator::Validate;

use crate::{auth::AuthSession, routes::AppState};

use super::{sign_up::analyze_password, UserError};

#[derive(Validate, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewPassword {
    old_password: String,
    #[validate(custom(function = "analyze_password"))]
    #[schema(default = "1Q2w3e4r5t6Y!~")]
    new_password: String,
}

#[utoipa::path(
    post,
    tag="user",
    path="/user/password/change",
    responses(
        (status = StatusCode::OK, description = "password changed")
    )
)]
pub async fn change_password(
    auth_session: AuthSession,
    State(app): State<AppState>,
    Json(new_password): Json<NewPassword>,
) -> Result<Json<Value>, UserError> {
    if new_password.old_password == new_password.new_password {
        return Err(UserError::SamePassword);
    };
    new_password.validate().map_err(UserError::Validation)?;

    let user = auth_session.user.ok_or(UserError::NotFound)?;

    task::spawn_blocking(move || verify_password(new_password.old_password, &user.password_hash))
        .await
        .context("non-blocking thread error")??;

    let pool = app.pool;
    let password_hash = task::spawn_blocking(|| generate_hash(new_password.new_password))
        .await
        .context("problem generating password hash")?;

    let now = OffsetDateTime::now_utc();
    sqlx::query!(
        "update users 
        set 
        password_hash = $1,
        last_updated = $2
        where 
        username = $3",
        password_hash,
        now,
        user.username
    )
    .execute(&pool)
    .await
    .context("error inserting new password into database")?;

    Ok(Json(json!({"message":"password successfully changed"})))
}

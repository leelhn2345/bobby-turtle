use anyhow::anyhow;
use axum::{extract::State, Json};
use chrono::Utc;
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
    old_password: String,
    #[validate(custom(function = "analyze_password"))]
    #[schema(default = "1Q2w3e4r5t6Y!~")]
    new_password: String,
}

#[utoipa::path(
    put,
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
    if new_password.old_password == new_password.new_password {
        return Err(UserError::SamePassword);
    };
    let curr_user = auth_session.clone().user;

    let Some(user) = curr_user else {
        return Err(anyhow!("no user recognized in current auth session").into());
    };

    let username = user.username;
    let login_creds = LoginCredentials::new(&username, &new_password.old_password);

    let user_exists = auth_session.authenticate(login_creds).await?;

    if user_exists.is_none() {
        return Err(UserError::InvalidCredentials);
    }

    new_password.validate()?;

    let password_hash = task::spawn_blocking(|| generate_hash(new_password.new_password)).await?;
    let now = Utc::now();
    sqlx::query!(
        "update users 
        set 
        password_hash = $1,
        last_updated = $2
        where 
        username = $3",
        password_hash,
        now,
        username
    )
    .execute(&pool)
    .await?;

    Ok(Json(json!({"message":"password successfully changed"})))
}

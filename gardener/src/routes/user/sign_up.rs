use axum::{extract::State, Json};
use password_auth::generate_hash;
use passwords::analyzer;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio::task;
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

use crate::auth::{AuthSession, AuthenticatedUser, PermissionLevel};

use super::{User, UserError};

#[derive(Deserialize, Validate, ToSchema)]
pub struct NewUser {
    #[serde(flatten)]
    user_info: User,

    #[validate(custom(function = "analyze_password"))]
    #[schema(default = "1Q2w3e4r5t!~")]
    password: String,
}

pub fn analyze_password(password: &str) -> Result<(), ValidationError> {
    let analyzed = analyzer::analyze(password);

    if analyzed.numbers_count() == 0 {
        return Err(ValidationError::new("no number in password"));
    }

    if analyzed.lowercase_letters_count() == 0 {
        return Err(ValidationError::new("no lowercase characters in password"));
    }

    if analyzed.uppercase_letters_count() == 0 {
        return Err(ValidationError::new("no uppercase characters in password"));
    }

    if analyzed.symbols_count() == 0 {
        return Err(ValidationError::new("no special characters in password"));
    }
    Ok(())
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

    let password_hash = task::spawn_blocking(|| generate_hash(new_user.password)).await?;

    let user = new_user.user_info;

    sqlx::query!(
        r#"
        insert into users 
        (user_id, username, password_hash, first_name, 
         last_name, joined_at, last_updated, permission_level)
        values 
        ($1,$2,$3,$4,
         $5,$6,$7,$8)
        "#,
        user.user_id,
        user.username,
        password_hash,
        user.first_name,
        user.last_name,
        user.joined_at,
        user.last_updated,
        user.permission_level as PermissionLevel
    )
    .execute(&pool)
    .await?;

    let auth_user = AuthenticatedUser::new(user.user_id, password_hash);

    auth_session.login(&auth_user).await?;

    Ok(Json(json!({"message":"user successfully created"})))
}

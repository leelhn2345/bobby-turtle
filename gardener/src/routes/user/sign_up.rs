use anyhow::Context;
use axum::{extract::State, Json};
use password_auth::generate_hash;
use passwords::analyzer;
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng, Rng,
};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{Postgres, Transaction};
use tokio::task;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::{auth::PermissionLevel, routes::AppState};

use super::{User, UserError};

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

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    Alphanumeric.sample_string(&mut rng, 25)
}

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
    path="/user/sign-up",
    responses(
        (status = StatusCode::CREATED, description = "user successfully registered"),
        (status = StatusCode::CONFLICT, description = "username is taken"),
        (status = StatusCode::UNPROCESSABLE_ENTITY, description = "validation error"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "internal server error")
    )
)]
#[tracing::instrument(skip_all, fields(username = new_user.user_info.username))]
pub async fn register_new_user(
    State(app): State<AppState>,
    Json(new_user): Json<NewUser>,
) -> Result<(StatusCode, Json<Value>), UserError> {
    new_user.validate().map_err(UserError::Validation)?;

    let mut tx = app
        .pool
        .begin()
        .await
        .context("failed to acquire postgres connection from pool")?;

    let exists = check_user_exists(&mut tx, &new_user.user_info.username)
        .await
        .context("problem checking if user exists")?;

    if exists {
        return Err(UserError::UsernameTaken);
    }

    let user_id = new_user.user_info.user_id;

    let password_hash = task::spawn_blocking(|| generate_hash(new_user.password))
        .await
        .context("error generating password hash in non-blocking thread")?;

    create_user(&mut tx, new_user.user_info.clone(), password_hash)
        .await
        .context("failed to insert new user in the database")?;

    let token = generate_subscription_token();
    store_token(&mut tx, user_id, &token)
        .await
        .context("failed to store confirmation token for new user")?;

    let name = match new_user.user_info.last_name {
        Some(last_name) => format!("{} {}", new_user.user_info.first_name, last_name),
        None => new_user.user_info.first_name,
    };
    app.email_client
        .send_confirmation_email(name, new_user.user_info.username, token)
        .await
        .context("error sending confirmation email")?;

    tx.commit()
        .await
        .context("failed to commit sql transaction to insert new user")?;

    Ok((
        StatusCode::CREATED,
        Json(json!({"message":"user successfully created"})),
    ))
}

async fn store_token(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO verification_tokens (verification_token, user_id)
    VALUES ($1, $2)
        "#,
        token,
        user_id,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
async fn create_user(
    tx: &mut Transaction<'_, Postgres>,
    user: User,
    password_hash: String,
) -> Result<(), sqlx::Error> {
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
    .execute(&mut **tx)
    .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn check_user_exists(
    tx: &mut Transaction<'_, Postgres>,
    username: &str,
) -> Result<bool, sqlx::Error> {
    let user_exists = sqlx::query_scalar!(
        "select exists(
            select 1 from users where username = $1
        )",
        username
    )
    .fetch_one(&mut **tx)
    .await?;

    let Some(exists) = user_exists else {
        return Err(sqlx::Error::RowNotFound);
    };

    Ok(exists)
}

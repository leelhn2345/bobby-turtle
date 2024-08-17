use anyhow::Context;
use axum::extract::{Query, State};
use password_auth::generate_hash;
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};
use reqwest::StatusCode;
use serde::Deserialize;
use sqlx::{PgPool, Postgres, Transaction};
use time::OffsetDateTime;
use tokio::task;
use utoipa::IntoParams;
use uuid::Uuid;
use validator::Validate;

use crate::routes::AppState;

use super::{sign_up::analyze_password, UserError};

#[derive(Validate, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct NewPasswordQuery {
    #[validate(custom(function = "analyze_password"))]
    new_password: String,
    reset_token: String,
}
#[utoipa::path(
    post,
    tag = "user",
    params(
        NewPasswordQuery
    ),
    path = "/user/password/reset-confirm",
    responses(
        (status=StatusCode::ACCEPTED, description="password changed"),
        (status=StatusCode::UNPROCESSABLE_ENTITY, description="password doesn't conform to valid password schema"),
        (status=StatusCode::INTERNAL_SERVER_ERROR, description="internal server error"),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn password_reset_confirm(
    State(app): State<AppState>,
    Query(query): Query<NewPasswordQuery>,
) -> Result<(), UserError> {
    query.validate().map_err(UserError::Validation)?;
    let mut tx = app
        .pool
        .begin()
        .await
        .context("failed to acquire postgres connection from pool")?;

    let user_id = get_user_id_from_token(&mut tx, &query.reset_token)
        .await
        .context("failed to get data from database")?
        .ok_or(UserError::NotFound)?;

    let password_hash = task::spawn_blocking(|| generate_hash(query.new_password))
        .await
        .context("error generating password hash in non-blocking thread")?;

    insert_new_password_for_user(&mut tx, password_hash, user_id)
        .await
        .context("can't set new password for user")?;
    delete_reset_token(&mut tx, &query.reset_token)
        .await
        .context("can't delete reset token")?;
    tx.commit()
        .await
        .context("failed to commit sql transaction to insert new user")?;

    Ok(())
}

async fn delete_reset_token(
    tx: &mut Transaction<'_, Postgres>,
    token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "delete from reset_password_tokens where reset_token = $1",
        token
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_new_password_for_user(
    tx: &mut Transaction<'_, Postgres>,
    password_hash: String,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update users 
        set password_hash = $1
        where user_id = $2
        ",
        password_hash,
        user_id
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
async fn get_user_id_from_token(
    tx: &mut Transaction<'_, Postgres>,
    token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let res = sqlx::query!(
        "select user_id from reset_password_tokens where reset_token = $1",
        token
    )
    .fetch_optional(&mut **tx)
    .await?;
    Ok(res.map(|x| x.user_id))
}

#[derive(Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct TokenValidityQuery {
    reset_token: String,
}

#[utoipa::path(
    get,
    tag="user",
    path="/user/password/reset",
    params(
        TokenValidityQuery
    ),
    responses(
        (status=StatusCode::OK,description="reset token is still valid"),
        (status=StatusCode::GONE,description="reset token is expired"),
        (status=StatusCode::NOT_FOUND,description="reset token not found")
    )
)]
#[tracing::instrument(skip_all)]
pub async fn check_reset_token_validity(
    State(app): State<AppState>,
    Query(query): Query<TokenValidityQuery>,
) -> Result<(), UserError> {
    let mut tx = app
        .pool
        .begin()
        .await
        .context("can't get connection to db")?;

    let expiry_datetime = get_password_reset_token_expiry(&mut tx, &query.reset_token)
        .await
        .context("failed to retreive expiry datetime with associated reset token")?
        .ok_or(UserError::NotFound)?;

    if OffsetDateTime::now_utc() >= expiry_datetime {
        delete_reset_token(&mut tx, &query.reset_token)
            .await
            .context("can't delete invalid token")?;
        tx.commit()
            .await
            .context("failed to commit transaction to delete token")?;
        return Err(UserError::ExpiredToken);
    }
    Ok(())
}

async fn get_password_reset_token_expiry(
    tx: &mut Transaction<'_, Postgres>,
    reset_token: &str,
) -> Result<Option<OffsetDateTime>, sqlx::Error> {
    let res = sqlx::query!(
        "select expires from reset_password_tokens where reset_token = $1",
        reset_token
    )
    .fetch_optional(&mut **tx)
    .await?;
    Ok(res.map(|x| x.expires))
}

#[derive(Validate, Deserialize, IntoParams)]
pub struct PasswordResetQuery {
    #[validate(email)]
    email: String,
}

#[utoipa::path(
    post,
    tag = "user",
    path = "/user/password/reset",
    params(
        PasswordResetQuery
    ),
    responses(
        (status=StatusCode::ACCEPTED, description = "password reset token generated and email sent."),
        (status=StatusCode::NOT_FOUND, description = "user not found"),
        (status=StatusCode::INTERNAL_SERVER_ERROR, description = "internal server error")
    )
)]
#[tracing::instrument(skip_all)]
pub async fn password_reset(
    State(app): State<AppState>,
    Query(query): Query<PasswordResetQuery>,
) -> Result<StatusCode, UserError> {
    let pool = app.pool;

    let user_id = get_user_id_from_email(&pool, &query.email)
        .await
        .context("failed to retrieve user_id associated with username/email")?
        .ok_or(UserError::NotFound)?;

    let reset_token = generate_reset_token();

    insert_token_into_reset_password_table(&pool, user_id, &reset_token)
        .await
        .context("failed to insert reset token")?;

    app.email_client
        .send_password_reset_email(query.email, reset_token)
        .await
        .context("failed to send email")?;
    Ok(StatusCode::ACCEPTED)
}

pub async fn get_user_id_from_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let res = sqlx::query!("select user_id from users where username = $1", email)
        .fetch_optional(pool)
        .await?;
    Ok(res.map(|x| x.user_id))
}

pub async fn insert_token_into_reset_password_table(
    pool: &PgPool,
    user_id: Uuid,
    token: &str,
) -> Result<(), sqlx::Error> {
    let expiry_datetime = OffsetDateTime::now_utc() + time::Duration::hours(1);
    sqlx::query!(
        "insert into reset_password_tokens (reset_token, user_id, expires)
        values ($1, $2, $3)
        on conflict (user_id) do update
        set reset_token = excluded.reset_token, expires = excluded.expires
        ",
        token,
        user_id,
        expiry_datetime
    )
    .execute(pool)
    .await?;
    Ok(())
}

fn generate_reset_token() -> String {
    let mut rng = thread_rng();
    Alphanumeric.sample_string(&mut rng, 25)
}

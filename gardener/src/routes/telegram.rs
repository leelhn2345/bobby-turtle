use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use axum_login::{login_required, predicate_required};
use chrono::Utc;
use serde::Serialize;
use teloxide::{requests::Requester, types::ChatId};
use turtle_bot::chatroom::{check_if_exists_and_inside, ChatRoomError};

use crate::auth::{AuthSession, Backend};

use super::AppState;

#[derive(thiserror::Error, Debug)]
pub enum TelegramError {
    #[error(transparent)]
    Chatroom(#[from] ChatRoomError),

    #[error("resource not found")]
    NotFound,

    #[error("user is unauthorized")]
    NotLoggedIn,

    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),

    #[error("token has expired")]
    ExpiredToken,
}

impl IntoResponse for TelegramError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (status_code, msg) = match self {
            Self::NotLoggedIn => (StatusCode::UNAUTHORIZED, "user is unauthorized".to_owned()),
            Self::NotFound => (StatusCode::NOT_FOUND, "resource(s) not found".to_owned()),
            Self::ExpiredToken => (StatusCode::GONE, "token has expired".to_owned()),
            Self::Chatroom(e) => {
                tracing::error!("{e:#?}");
                (
                    StatusCode::NOT_FOUND,
                    "room does not exist or bot is not in chat".to_owned(),
                )
            }
            Self::UnknownError(e) => {
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

/// sends message to telegram chat
#[utoipa::path(
    post,
    tag = "telegram",
    path = "/telegram/message/{id}",
    params(
        ("id", description="id of chat")
    ),
    request_body(content=String, description="message to send"),
    responses(
        (status = StatusCode::OK, description = "message sent"),
        (status = StatusCode::NOT_FOUND, description = "room does not exist or bot is not in chat")
    )
)]
pub async fn send_tele_msg(
    State(app): State<AppState>,
    Path(chat_id): Path<i64>,
    msg: String,
) -> Result<(), TelegramError> {
    let mut tx = app
        .pool
        .begin()
        .await
        .context("can't get postgres connection")?;
    check_if_exists_and_inside(&mut tx, chat_id).await?;
    tx.commit().await.context("can't commit transaction")?;
    app.bot
        .send_message(ChatId(chat_id), msg)
        .await
        .context("failed sending message to telegram server")?;
    Ok(())
}

#[utoipa::path(
    post,
    tag = "telegram",
    path = "/telegram/verify-user/{token}",
    params(
        ("token", description = "verification token")
    ),
    responses(
        (status = 202, description = "verification successful"),
        (status = 401, description = "user is not logged in"),
        (status = 404, description = "user is not pending verification"),
        (status = 410, description = "token has expired"),
        (status = 505, description = "internal server error")
    )
)]
#[tracing::instrument(skip_all)]
pub async fn verify_telegram_user(
    auth_session: AuthSession,
    State(app): State<AppState>,
    Path(token): Path<String>,
) -> Result<StatusCode, TelegramError> {
    println!("hello");
    let mut tx = app
        .pool
        .begin()
        .await
        .context("failed to get postgres connection")?;

    let expiry_record = sqlx::query!(
        "
        select expiry
        from telegram_tokens
        where telegram_token = $1
        ",
        token
    )
    .fetch_optional(&mut *tx)
    .await
    .context("failed to check if token exists")?
    .ok_or(TelegramError::NotFound)?;

    let now = Utc::now();

    if now >= expiry_record.expiry {
        let pool = app.pool;
        sqlx::query!(
            "
            delete from telegram_tokens where telegram_token = $1
            ",
            token
        )
        .execute(&pool)
        .await
        .context("failed to delete token")?;
        return Err(TelegramError::ExpiredToken);
    }

    let user_id = auth_session.user.ok_or(TelegramError::NotLoggedIn)?.user_id;

    sqlx::query!(
        "
        insert into telegram_users (telegram_user_id, telegram_username, user_id,joined_at)
        select telegram_user_id, telegram_username, $1, $2
        from telegram_tokens
        where telegram_token = $3
        ",
        user_id,
        now,
        token
    )
    .execute(&mut *tx)
    .await
    .context("failed to insert new telegram_users row")?;

    sqlx::query!(
        "delete from telegram_tokens where telegram_token = $1",
        token
    )
    .execute(&mut *tx)
    .await
    .context("failed to delete token")?;

    tx.commit().await.context("failed to commit transaction")?;
    Ok(StatusCode::ACCEPTED)
}

/// only allow apis for verified users
#[allow(clippy::unused_async)]
async fn is_telegram_user(auth_session: AuthSession) -> bool {
    let user = auth_session.user;
    let Some(user) = user else { return false };
    user.telegram_verified
}

pub fn tele_router() -> Router<AppState> {
    Router::new()
        .route("/message/:chat_id", post(send_tele_msg))
        .route_layer(predicate_required!(
            is_telegram_user,
            (StatusCode::UNAUTHORIZED, "not verified").to_owned()
        ))
        .route(
            "/verify-user/:token",
            post(verify_telegram_user).layer(login_required!(Backend)),
        )
}

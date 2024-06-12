use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_login::{login_required, predicate_required};
use chrono::Utc;
use serde::Serialize;
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{ChatId, ParseMode},
    ApiError, RequestError,
};
use utoipa::ToSchema;

use crate::auth::{AuthSession, Backend};

use super::AppState;

#[derive(thiserror::Error, Debug)]
pub enum TelegramError {
    #[error(transparent)]
    TeloxideError(#[from] teloxide::RequestError),

    #[error("resource not found")]
    NotFound,

    #[error("bot not in chat")]
    BotNotInChat,

    #[error("user is unauthorized")]
    NotLoggedIn,

    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),

    #[error("token has expired")]
    ExpiredToken,

    #[error("User has left chatroom")]
    UserNotInChat,
}

impl IntoResponse for TelegramError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (status_code, msg) = match self {
            Self::BotNotInChat => (StatusCode::FORBIDDEN, "bot no longer in chat".to_owned()),
            Self::UserNotInChat => (StatusCode::FORBIDDEN, "user is not in chat".to_owned()),
            Self::NotLoggedIn => (StatusCode::UNAUTHORIZED, "user is unauthorized".to_owned()),
            Self::NotFound => (StatusCode::NOT_FOUND, "resource(s) not found".to_owned()),
            Self::ExpiredToken => (StatusCode::GONE, "token has expired".to_owned()),
            Self::TeloxideError(e) => {
                tracing::error!("{e:#?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_owned(),
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
#[allow(deprecated)]
pub async fn send_tele_msg(
    auth_session: AuthSession,
    State(app): State<AppState>,
    Path(chat_id): Path<i64>,
    msg: String,
) -> Result<(), TelegramError> {
    let pool = app.pool;
    let user_id = auth_session
        .user
        .context("user is using protected api")?
        .user_id;

    let exists = sqlx::query_scalar!(
        "select exists (
        select * from telegram_whisperers as a
        inner join telegram_users as b on b.user_id = $1
        where a.telegram_chat_id = $2
        )",
        user_id,
        chat_id
    )
    .fetch_one(&pool)
    .await
    .context("error checking if user and chatroom is in database")?
    .context("query scalar returns None")?;

    if !exists {
        return Err(TelegramError::UserNotInChat);
    }

    let msg_result = app
        .bot
        .send_message(ChatId(chat_id), msg)
        .parse_mode(ParseMode::Markdown)
        .await
        .map_err(|e| match e {
            RequestError::Api(ApiError::BotKicked | ApiError::BotKickedFromSupergroup) => {
                TelegramError::BotNotInChat
            }
            _ => e.into(),
        });

    if let Err(TelegramError::BotNotInChat) = msg_result {
        sqlx::query!(
            "delete from telegram_whisperers where telegram_chat_id = $1",
            chat_id
        )
        .execute(&pool)
        .await
        .context("bot has been kicked. can't delete chat room from database")?;
    }

    msg_result?;
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

#[utoipa::path(
    get,
    path = "/telegram/check-user",
    tag="telegram",
    responses(
        (status=200, body=Option<bool>)
    )
)]
/// check for user's telegram status
///
/// 1. `true` - user is verified
/// 2. `false` - user is not registered
/// 3. none - user is not logged in
async fn check_if_verified(auth_session: AuthSession) -> Json<Option<bool>> {
    let user = auth_session.user;
    let res = match user {
        None => None,
        Some(user) => Some(user.telegram_verified),
    };
    Json(res)
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChatroomInfo {
    telegram_chat_id: i64,
    title: Option<String>,
}

#[utoipa::path(
    get,
    path="/telegram/chats-available",
    tag="telegram",
    responses(
        (status = 200, body = Vec<ChatroomInfo>, description = "list of available chats user can whisper to"),
        (status = 505, description = "internal server error")
    )
)]
#[tracing::instrument(skip_all)]
async fn chats_available(
    State(app): State<AppState>,
    auth_session: AuthSession,
) -> Result<Json<Vec<ChatroomInfo>>, TelegramError> {
    let pool = app.pool;

    let user_id = auth_session.user.context("no user in session")?.user_id;

    let chats = sqlx::query_as!(
        ChatroomInfo,
        r#"select
        c.telegram_chat_id,
        d.title
        from
        users as a
        inner join telegram_users as b on b.user_id = a.user_id
        inner join telegram_whisperers as c on b.telegram_user_id = c.telegram_user_id
        inner join chatrooms as d on c.telegram_chat_id = d.id and d.is_group is true
        where
        a.user_id = $1
        "#,
        user_id
    )
    .fetch_all(&pool)
    .await
    .context("can't retrieve chat info")?;

    Ok(Json(chats))
}

pub fn tele_router() -> Router<AppState> {
    let verified_user_routes = Router::new()
        .route("/message/:chat_id", post(send_tele_msg))
        .route_layer(predicate_required!(
            is_telegram_user,
            (StatusCode::UNAUTHORIZED, "not verified").to_owned()
        ));

    let need_log_in_routes = Router::new()
        .route("/verify-user/:token", post(verify_telegram_user))
        .route("/chats-available", get(chats_available))
        .route_layer(login_required!(Backend));

    Router::new()
        .route("/check-user", get(check_if_verified))
        .merge(verified_user_routes)
        .merge(need_log_in_routes)
}

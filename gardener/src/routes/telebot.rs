use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::Serialize;
use teloxide::{requests::Requester, types::ChatId, RequestError};
use turtle_bot::chatroom::{check_if_exists_and_inside, ChatRoomError};

use super::AppState;

#[derive(thiserror::Error, Debug)]
pub enum BotError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    TelegramRequest(#[from] RequestError),

    #[error(transparent)]
    Chatroom(#[from] ChatRoomError),
}

impl IntoResponse for BotError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (status_code, msg) = match self {
            Self::TelegramRequest(e) => {
                tracing::error!("{e:#?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "error sending request to telegram".to_owned(),
                )
            }
            Self::Sqlx(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_owned(),
            ),
            Self::Chatroom(e) => {
                tracing::error!("{e:#?}");
                (
                    StatusCode::NOT_FOUND,
                    "room does not exist or bot is not in chat".to_owned(),
                )
            }
        };
        (status_code, Json(ErrorResponse { message: msg })).into_response()
    }
}

/// sends message to telegram chat
#[utoipa::path(
    post,
    tag = "bot",
    path = "/bot/message/{id}",
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
) -> Result<(), BotError> {
    let mut tx = app.pool.begin().await?;
    check_if_exists_and_inside(&mut tx, chat_id).await?;
    app.bot.send_message(ChatId(chat_id), msg).await?;
    Ok(())
}

pub fn bot_router() -> Router<AppState> {
    Router::new().route("/message/:chat_id", post(send_tele_msg))
}

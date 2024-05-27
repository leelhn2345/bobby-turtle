use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::Serialize;
use teloxide::{requests::Requester, types::ChatId, RequestError};

use super::AppState;

#[derive(thiserror::Error, Debug)]
pub enum BotError {
    #[error(transparent)]
    TelegramRequest(#[from] RequestError),
}

impl IntoResponse for BotError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (status_code, msg) = match self {
            BotError::TelegramRequest(e) => {
                tracing::error!("{e:#?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "error sending request to telegram".to_owned(),
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
        (status = StatusCode::OK, description = "message sent")
    )
)]
pub async fn send_tele_msg(
    State(app): State<AppState>,
    Path(chat_id): Path<i64>,
    msg: String,
) -> Result<(), BotError> {
    app.bot.send_message(ChatId(chat_id), msg).await?;
    Ok(())
}

pub fn bot_router() -> Router<AppState> {
    Router::new().route("/message/:chat_id", post(send_tele_msg))
}

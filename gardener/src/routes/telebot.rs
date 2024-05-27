use axum::extract::State;
use teloxide::{requests::Requester, types::ChatId};

use super::AppState;

#[utoipa::path(post, tag = "bot", path = "/bot-test")]
pub async fn send_tele_msg(State(app): State<AppState>) {
    app.bot
        .send_message(ChatId(220_272_763), "cefe")
        .await
        .unwrap();
}

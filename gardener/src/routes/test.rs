use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use tower_sessions::{
    cookie::{Cookie, Expiration},
    Session,
};

use super::AppState;

#[utoipa::path(get, path = "/test/cookie_handler", tag = "test")]
pub async fn cookie_handler(jar: CookieJar) -> Result<(CookieJar, String), StatusCode> {
    let zz = Cookie::build(("turtle", "check me in dev tools")).expires(Expiration::Session);
    Ok((jar.add(zz), "check the damn cookie".to_string()))
}

const COUNTER_KEY: &str = "counter";

#[derive(Serialize, Deserialize, Default)]
struct Counter(usize);

#[utoipa::path(get, path = "/test/session_handler", tag = "test")]
pub async fn session_handler(session: Session) -> impl IntoResponse {
    let counter: Counter = session.get(COUNTER_KEY).await.unwrap().unwrap_or_default();
    session.insert(COUNTER_KEY, counter.0 + 1).await.unwrap();
    format!("Current count: {}", counter.0)
}

pub fn test_router() -> Router<AppState> {
    Router::new()
        .route("/cookie_handler", get(cookie_handler))
        .route("/session_handler", get(session_handler))
}

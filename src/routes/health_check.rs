use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub async fn root() -> Response {
    "hello world!".into_response()
}


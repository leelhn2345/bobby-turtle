use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Root URL
#[utoipa::path(
    get,
    tag="/",
    path="/",
    responses(
        ( status=StatusCode::OK,description="hello world!")
    )
)]
pub async fn root() -> Response {
    "hello world!".into_response()
}

/// health check
///
/// checks if the app is functioning.
#[utoipa::path(
    get,
    tag="/",
    path="/health_check",
    responses(
        (status = 200, description = "App is healthy.")
    )
)]
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

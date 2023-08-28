use axum::http::StatusCode;

#[utoipa::path(
    get,
    path="/health_check",
    responses(
        (status=200,description = "Server is running")
    )
)]
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

use axum::{routing::get, Router};

use crate::routes::{health_check, root};

pub async fn start_app() {
    let app = Router::new()
        .route("/", get(root))
        .route("/health_check", get(health_check));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("port is unavailable");

    axum::serve(listener, app).await.unwrap();
}

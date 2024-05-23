mod routes;

use axum::{routing::get, Router, ServiceExt};
use gaia::Settings;
use sqlx::PgPool;

use crate::routes::app_router;

pub async fn start_app(settings: Settings, pool: PgPool) {
    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.web_port
    );

    // debug only runs in local dev environment
    tracing::debug!(
        "starting gardener app @ http://localhost:{}",
        settings.application.web_port
    );

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("can't bind address to tcp listener");

    let app_router = app_router(pool);

    axum::serve(listener, app_router.into_make_service())
        .await
        .expect("error starting axum app");
}

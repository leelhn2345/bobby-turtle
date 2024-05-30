mod auth;
mod routes;

use axum_login::tower_sessions::ExpiredDeletion;
use gaia::Settings;
use sqlx::PgPool;
use teloxide::Bot;
use tower_sessions_sqlx_store::PostgresStore;

use crate::routes::app_router;

#[tracing::instrument(skip_all, name = "gardener")]
pub async fn start_app(settings: Settings, pool: PgPool, bot: Bot) {
    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.web_port
    );

    // debug only runs in local dev environment
    tracing::debug!(
        "starting gardener app @ http://localhost:{}",
        settings.application.web_port
    );

    let session_store = PostgresStore::new(pool.clone());

    session_store
        .migrate()
        .await
        .expect("can't migrate session schema");

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("can't bind address to tcp listener");

    tokio::task::spawn(
        session_store
            .clone()
            // clean every 30 mins
            .continuously_delete_expired(tokio::time::Duration::from_secs(1800)),
    );

    let app_router = app_router(&settings.application, session_store, pool, bot);

    axum::serve(listener, app_router.into_make_service())
        .await
        .expect("error starting axum app");
}

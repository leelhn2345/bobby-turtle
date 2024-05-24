mod auth;
mod routes;

use axum_login::tower_sessions::ExpiredDeletion;
use gaia::{environment::Environment, Settings};
use sqlx::PgPool;
use tokio::{signal, task::AbortHandle};
use tower_sessions_sqlx_store::PostgresStore;
use turtle_bot::start_bot;

use crate::routes::app_router;

#[tracing::instrument(skip_all, name = "gardener")]
pub async fn start_app(env: Environment, settings: Settings, pool: PgPool) {
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

    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    let bot_app = tokio::spawn(start_bot(env, settings.clone(), pool.clone()));

    let app_router = app_router(session_store, pool);

    axum::serve(listener, app_router.into_make_service())
        .with_graceful_shutdown(shutdown_signal(
            deletion_task.abort_handle(),
            bot_app.abort_handle(),
        ))
        .await
        .expect("error starting axum app");

    match deletion_task.await {
        Ok(Ok(())) => {
            tracing::info!("deletion_task has exited");
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "deletion_task failed to start",
            );
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "deletion_task task failed to complete",
            );
        }
    };
}

async fn shutdown_signal(
    deletion_task_abort_handle: AbortHandle,
    bot_app_abort_handle: AbortHandle,
) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install ctrl+c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        () = ctrl_c => {
            deletion_task_abort_handle.abort();
            bot_app_abort_handle.abort();
        },
        () = terminate=>{
            deletion_task_abort_handle.abort();
            bot_app_abort_handle.abort();
        }
    }
}

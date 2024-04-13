use axum::{routing::get, Router};
use utoipa::OpenApi;
use utoipa_auto_discovery::utoipa_auto_discovery;
use utoipa_swagger_ui::SwaggerUi;

use crate::{routes::health_check, settings::Settings};

#[utoipa_auto_discovery(paths = "
    ( health_check => ./src/routes/health_check.rs );
")]
#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

#[tracing::instrument(
    level = "debug" 
    name = "starting bot app"
    skip_all
)]
pub async fn start_app(settings: Settings) {
    let app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/docs.json", ApiDoc::openapi()))
        .route("/", get(health_check::root))
        .route("/health_check", get(health_check::health_check));

    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.port
    );
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("port is unavailable");

    tracing::debug!("app is running on http://localhost:{}", {
        settings.application.port
    });

    axum::serve(listener, app).await.unwrap();
}

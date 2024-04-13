use axum::{routing::get, Router};
use utoipa::OpenApi;
use utoipa_auto_discovery::utoipa_auto_discovery;
use utoipa_swagger_ui::SwaggerUi;

use crate::routes::health_check;

#[utoipa_auto_discovery(paths = "
    ( health_check => ./src/routes/health_check.rs );
")]
#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

pub async fn start_app() {
    let app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/docs.json", ApiDoc::openapi()))
        .route("/", get(health_check::root))
        .route("/health_check", get(health_check::health_check));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("port is unavailable");

    axum::serve(listener, app).await.unwrap();
}

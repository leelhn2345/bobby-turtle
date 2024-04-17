pub mod health_check;

use axum::{body::Body, http::Request, routing::get, Router};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_auto_discovery::utoipa_auto_discovery;
use utoipa_swagger_ui::SwaggerUi;

#[utoipa_auto_discovery(paths = "
    ( health_check => ./src/routes/health_check.rs );
")]
#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

pub fn app_router(router: Router) -> Router {
    let trace_layer = ServiceBuilder::new().layer(TraceLayer::new_for_http().make_span_with(
        |request: &Request<Body>| {
            let req_id = uuid::Uuid::new_v4();
            tracing::info_span!(
                "request",
                method = tracing::field::display(request.method()),
                uri = tracing::field::display(request.uri()),
                req_id = tracing::field::display(req_id)
            )
        },
    ));
    router
        .merge(SwaggerUi::new("/docs").url("/docs.json", ApiDoc::openapi()))
        .route("/", get(health_check::root))
        .route("/health_check", get(health_check::health_check))
        .layer(trace_layer)
}

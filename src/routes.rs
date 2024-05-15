pub mod health_check;
mod resume;

use axum::{body::Body, http::Request, routing::get, Router};
use sqlx::PgPool;
use teloxide::Bot;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipauto::utoipauto;

#[utoipauto]
#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

pub fn app_router(router: Router, pool: PgPool, bot: Bot) -> Router {
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

    router.merge(
        Router::new()
            .merge(SwaggerUi::new("/docs").url("/docs.json", ApiDoc::openapi()))
            .route("/resume", get(resume::resume_details))
            .with_state(pool)
            .with_state(bot)
            .layer(trace_layer)
            .route("/", get(health_check::root))
            .route("/health_check", get(health_check::health_check)),
    )
}

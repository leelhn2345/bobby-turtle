mod health_check;
mod resume;
mod user;

use std::time::Duration;

use axum::{
    body::Body,
    http::{Request, Response},
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::Span;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;
use utoipauto::utoipauto;

#[utoipauto]
#[derive(OpenApi)]
#[openapi(modifiers(&SecurityAddon))]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "cookieAuth",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("HAHAID"))),
            );
        }
    }
}

pub fn app_router(pool: PgPool) -> Router {
    let trace_layer = ServiceBuilder::new().layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request<Body>| {
                let request_id = uuid::Uuid::new_v4();
                tracing::info_span!(
                    "request",
                    method = tracing::field::display(request.method()),
                    uri = tracing::field::display(request.uri()),
                    version = tracing::field::debug(request.version()),
                    request_id = tracing::field::display(request_id),
                    latency = tracing::field::Empty,
                    status_code = tracing::field::Empty,
                )
            })
            .on_response(
                |response: &Response<Body>, latency: Duration, span: &Span| {
                    span.record("status_code", &tracing::field::display(response.status()));
                    span.record("latency", &tracing::field::debug(latency));
                    // trace here
                },
            ),
    );

    Router::new()
        .merge(SwaggerUi::new("/docs").url("/docs.json", ApiDoc::openapi()))
        .route("/resume", get(resume::resume_details))
        .route("/sign_up", post(user::sign_up::register_new_user))
        .route("/login", post(user::login::login))
        .with_state(pool)
        .layer(trace_layer)
        .route("/", get(health_check::root))
        .route("/health_check", get(health_check::health_check))
}

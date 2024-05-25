mod health_check;
mod resume;
pub mod user;

use axum::{
    body::Body,
    http::{Request, Response},
    routing::{get, post},
    Router,
};
use axum_login::{login_required, AuthManagerLayerBuilder};
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tower_sessions::{
    cookie::{time::Duration, Key},
    Expiry, SessionManagerLayer,
};
use tower_sessions_sqlx_store::PostgresStore;
use tracing::Span;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;
use utoipauto::utoipauto;

use crate::auth::Backend;

#[utoipauto(paths = "./gardener/src")]
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

pub fn app_router(session_store: PostgresStore, pool: PgPool) -> Router {
    let trace_layer = TraceLayer::new_for_http()
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
            |response: &Response<Body>, latency: std::time::Duration, span: &Span| {
                span.record("status_code", &tracing::field::display(response.status()));
                span.record("latency", &tracing::field::debug(latency));
                // add tracing below here
            },
        );

    let key = Key::generate();

    let session_layer = SessionManagerLayer::new(session_store)
        // .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)))
        .with_name("gardener.id")
        .with_signed(key);

    let backend = Backend::new(pool.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let layers = ServiceBuilder::new().layer(trace_layer).layer(auth_layer);

    Router::new()
        .merge(SwaggerUi::new("/docs").url("/docs.json", ApiDoc::openapi()))
        .merge(
            // login required routes here
            Router::new()
                .route("/logout", get(user::logout))
                .route(
                    "/change-password",
                    post(user::change_password::change_password),
                )
                .route_layer(login_required!(Backend)),
        )
        .route("/resume", get(resume::resume_details))
        .route("/sign_up", post(user::sign_up::register_new_user))
        .route("/login", post(user::login))
        .with_state(pool)
        .layer(layers)
        .route("/", get(health_check::root))
        .route("/health_check", get(health_check::health_check))
}

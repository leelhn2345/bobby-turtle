mod health_check;
mod resume;
mod telebot;
pub mod user;

use axum::{
    body::Body,
    http::{header::CONTENT_TYPE, Request, Response, StatusCode},
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use axum_login::{predicate_required, AuthManagerLayerBuilder};
use sqlx::PgPool;
use teloxide::Bot;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tower_sessions::{
    cookie::{time::Duration, Cookie, Key, SameSite},
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

use crate::auth::{AuthSession, Backend, PermissionLevel};

use self::{telebot::bot_router, user::user_router};

#[utoipauto(paths = "./gardener/src")]
#[derive(OpenApi)]
#[openapi(modifiers(&SecurityAddon))]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "securityScheme1",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("scheme1cookie1"))),
            );
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
    bot: Bot,
}

impl AppState {
    pub fn new(pool: PgPool, bot: Bot) -> Self {
        Self { pool, bot }
    }
}

pub fn app_router(session_store: PostgresStore, pool: PgPool, bot: Bot) -> Router {
    let cors_layer = CorsLayer::new()
        .allow_origin(["http://localhost:3000".parse().unwrap()])
        .allow_headers([CONTENT_TYPE])
        .allow_credentials(true);

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
                // useful if using bunyan trace format
            },
        );

    let key = Key::generate();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::weeks(2)))
        .with_same_site(SameSite::None)
        .with_name("gardener.id")
        .with_signed(key.clone());

    let backend = Backend::new(pool.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let layers = ServiceBuilder::new()
        .layer(trace_layer)
        .layer(auth_layer)
        .layer(cors_layer);

    let app_state = AppState::new(pool, bot);

    Router::new()
        .merge(SwaggerUi::new("/docs").url("/docs.json", ApiDoc::openapi()))
        .route("/resume", get(resume::resume_details))
        .route("/handler", get(cookie_handler))
        .nest("/user", user_router())
        .nest("/bot", bot_router())
        .with_state(app_state)
        .layer(layers)
        .route("/", get(health_check::root))
        .route("/health_check", get(health_check::health_check))
        .fallback(|| async { (StatusCode::NOT_FOUND, "invalid api") })
}

#[utoipa::path(get, path = "/handler", tag = "test")]
async fn cookie_handler(jar: CookieJar) -> Result<(CookieJar, String), StatusCode> {
    let zz = Cookie::build(("fefe", "CVEve"))
        .http_only(true)
        .same_site(SameSite::None)
        .max_age(Duration::weeks(2))
        .path("/")
        .secure(true);
    Ok((jar.add(zz), "check the damn cookie".to_string()))
}

#[allow(clippy::unused_async, dead_code)]
async fn is_admin(auth_session: AuthSession) -> bool {
    let Some(user) = auth_session.user else {
        return false;
    };

    if user.permission_level == PermissionLevel::Member {
        return false;
    }

    true
}

#[allow(clippy::unused_async, dead_code)]
async fn admin_routes() -> Router<AppState> {
    Router::new().route_layer(predicate_required!(is_admin, StatusCode::FORBIDDEN))
}

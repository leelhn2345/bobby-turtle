use axum::{routing, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod health_check;

use health_check::*;

#[derive(OpenApi)]
#[openapi(
    paths(health_check::health_check),
    tags((name = "polarbearr", description = "sample axum swagger integration"))
)]
pub struct ApiDoc;

pub fn setup_bot_router(bot_router: Router) -> Router {
    bot_router
        .merge(SwaggerUi::new("/docs").url("/docs.json", ApiDoc::openapi()))
        .route("/", routing::get(|| async { "Hello World!" }))
        .route("/health_check", routing::get(health_check))
}

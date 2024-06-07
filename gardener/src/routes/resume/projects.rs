use anyhow::Context;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::AppState;

#[derive(thiserror::Error, Debug)]
enum ProjectsError {
    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
}

impl IntoResponse for ProjectsError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }
        let (status_code, msg) = match self {
            Self::UnknownError(e) => {
                tracing::error!("{e:#?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_owned(),
                )
            }
        };
        (status_code, Json(ErrorResponse { message: msg })).into_response()
    }
}

#[derive(sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "publicity", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
enum Publicity {
    Public,
    Member,
}

#[derive(Serialize, ToSchema)]
pub struct ProjectSummary {
    id: i32,
    name: String,
    summary: String,
    url: Option<String>,
    publicity: Publicity,
}

#[utoipa::path(
    get,
    tag="resume",
    path="/resume/projects/summary",
    responses(
        (status = 200, body=ProjectSummary, description = "list of projects"),
        (status = 505, description = "internal server error"),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_projects_summary(
    State(app): State<AppState>,
) -> Result<Json<Vec<ProjectSummary>>, ProjectsError> {
    let pool = app.pool;
    let projects = sqlx::query_as!(
        ProjectSummary,
        r#"
        select 
        id, 
        name, 
        url, 
        summary,
        publicity as "publicity: Publicity"
        from project_demos
        order by id desc
        "#
    )
    .fetch_all(&pool)
    .await
    .context("can't retrieve projects data")
    .map(Json)?;
    Ok(projects)
}
pub fn project_router() -> Router<AppState> {
    Router::new().route("/summary", get(get_projects_summary))
}

use std::time::Instant;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use indexmap::IndexMap;
use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;

#[derive(thiserror::Error, Debug)]
pub enum AboutPageError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for AboutPageError {
    fn into_response(self) -> axum::response::Response {
        let msg = match self {
            Self::Sqlx(_) => "database error",
            Self::Unexpected(_) => "unknown error",
        };

        (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
    }
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResumeDetails {
    about_me: String,
    skills: Skills,
    job_experiences: Vec<JobExperience>,
    projects: Vec<Projects>,
}

#[derive(Serialize, ToSchema)]
pub struct Skills {
    languages: String,
    tools: String,
    frameworks: String,
    others: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JobExperience {
    company_name: String,
    company_url: String,
    jobs_in_company: Vec<JobDescription>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JobDescription {
    id: i32,
    job_title: String,
    time_span: String,
    description: Option<Vec<String>>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Projects {
    id: i32,
    project_name: String,
    project_url: String,
    description: Option<Vec<String>>,
}

/// resume details
///
/// gets data about resume
#[utoipa::path(
    get,
    tag="about",
    path="/resume",
    responses(
        (status = 200, body=ResumeDetails, description = "resume data")
    )
)]
#[tracing::instrument(skip_all)]
pub async fn resume_details(
    State(pool): State<PgPool>,
) -> Result<Json<ResumeDetails>, AboutPageError> {
    let now = Instant::now();
    let job_experiences = get_job_experience(&pool).await?;
    let projects = get_projects(&pool).await?;
    let about_me = get_about_me(&pool).await?;
    let skills = get_skills(&pool).await?;
    let resume = ResumeDetails {
        about_me,
        skills,
        job_experiences,
        projects,
    };
    let elapsed = now.elapsed().as_secs_f32();
    tracing::debug!("{elapsed}s has elapsed");

    Ok(Json(resume))
}

struct JobInfo {
    id: i32,
    company_name: String,
    company_url: String,
    job_title: String,
    time_span: String,
    description: Option<Vec<String>>,
}

async fn get_job_experience(pool: &PgPool) -> Result<Vec<JobExperience>, AboutPageError> {
    let job_info: Vec<JobInfo> = sqlx::query_as!(
        JobInfo,
        "select id, company_name, company_url,job_title,time_span,description 
        from job_experience 
        order by id desc"
    )
    .fetch_all(pool)
    .await?;

    let mut job_info_hashmap: IndexMap<String, JobExperience> = IndexMap::new();

    for info in job_info {
        job_info_hashmap
            .entry(info.company_name.clone())
            .and_modify(|e| {
                e.jobs_in_company.push(JobDescription {
                    id: info.id,
                    job_title: info.job_title.clone(),
                    time_span: info.time_span.clone(),
                    description: info.description.clone(),
                });
            })
            .or_insert(JobExperience {
                company_name: info.company_name,
                company_url: info.company_url,
                jobs_in_company: vec![JobDescription {
                    id: info.id,
                    job_title: info.job_title,
                    time_span: info.time_span,
                    description: info.description,
                }],
            });
    }
    Ok(job_info_hashmap.into_values().collect())
}

async fn get_projects(pool: &PgPool) -> Result<Vec<Projects>, AboutPageError> {
    let projects = sqlx::query_as!(
        Projects,
        "select
        id, project_name,project_url,description
        from projects
        order by id desc
        "
    )
    .fetch_all(pool)
    .await?;

    Ok(projects)
}

async fn get_about_me(pool: &PgPool) -> Result<String, AboutPageError> {
    let about_me = sqlx::query!("select about_me from about_me")
        .fetch_one(pool)
        .await?;
    Ok(about_me.about_me)
}

async fn get_skills(pool: &PgPool) -> Result<Skills, AboutPageError> {
    let skills = sqlx::query_as!(
        Skills,
        "select languages,tools,frameworks,others
        from job_skills"
    )
    .fetch_one(pool)
    .await?;

    Ok(skills)
}

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::ToSchema;
use validator::Validate;

use super::{analyze_password, UserError};

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct NewUserCredentials {
    #[validate(email)]
    #[schema(default = "user@email.com")]
    username: String,

    #[validate(length(min = 8, max = 20), custom(function = "analyze_password"))]
    #[schema(default = "1Q2w3e4r5t!~")]
    password: String,
    first_name: String,
    last_name: Option<String>,
}
/// new user
#[utoipa::path(
    post,
    tag="user",
    path="/sign_up",
    responses(
        (status = StatusCode::OK, description = "user successfully registered"),
        (status = StatusCode::CONFLICT, description = "username is taken"),
        (status = StatusCode::UNPROCESSABLE_ENTITY, description = "validation error"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "internal server error")
    )
)]
#[tracing::instrument(skip_all,fields(username = new_user_creds.username))]
pub async fn register_new_user(
    State(pool): State<PgPool>,
    Json(new_user_creds): Json<NewUserCredentials>,
) -> Result<StatusCode, UserError> {
    new_user_creds.validate()?;

    let user = sqlx::query!(
        "select from users where username = $1",
        new_user_creds.username
    )
    .fetch_optional(&pool)
    .await?;

    if user.is_some() {
        return Err(UserError::UsernameTaken);
    };

    let password = new_user_creds.password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password, &salt)
        .map_err(|_| {
            UserError::UnexpectedError(
                "couldn't turn new user's password into password hash".to_string(),
            )
        })?
        .to_string();

    sqlx::query!(
        "insert into users (user_id,username,password_hash)
        values ($1,$2,$3)",
        uuid::Uuid::new_v4(),
        new_user_creds.username,
        password_hash
    )
    .execute(&pool)
    .await?;

    Ok(StatusCode::CREATED)
}

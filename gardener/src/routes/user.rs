use axum::{http::StatusCode, response::IntoResponse, Json};
use passwords::analyzer;
use serde::Serialize;
use validator::{ValidationError, ValidationErrors};

pub mod login;
pub mod sign_up;

#[derive(thiserror::Error, Debug)]
pub enum UserError {
    #[error("username is taken")]
    UsernameTaken,

    #[error(transparent)]
    Validation(#[from] ValidationErrors),

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("{0}")]
    UnexpectedError(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for UserError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (status_code, msg) = match self {
            Self::UsernameTaken => (StatusCode::CONFLICT, "username is taken".to_owned()),
            Self::Validation(e) => {
                tracing::error!("{e:#?}");
                let fields: Vec<&str> = e.field_errors().into_keys().collect();
                let field_string = fields.join(", ");
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!("validation error in {field_string}"),
                )
            }
            Self::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "invalid credentials".to_owned())
            }

            Self::UnexpectedError(e) => {
                tracing::error!(e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_owned(),
                )
            }

            Self::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_owned(),
            ),
        };

        (status_code, Json(ErrorResponse { message: msg })).into_response()
    }
}

pub fn analyze_password(password: &str) -> Result<(), ValidationError> {
    let analyzed = analyzer::analyze(password);

    if analyzed.numbers_count() == 0 {
        return Err(ValidationError::new("no number in password"));
    }

    if analyzed.lowercase_letters_count() == 0 {
        return Err(ValidationError::new("no lowercase characters in password"));
    }

    if analyzed.uppercase_letters_count() == 0 {
        return Err(ValidationError::new("no uppercase characters in password"));
    }

    if analyzed.symbols_count() == 0 {
        return Err(ValidationError::new("no special characters in password"));
    }
    Ok(())
}

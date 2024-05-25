use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend};
use password_auth::verify_password;
use sqlx::PgPool;
use tokio::task;
use uuid::Uuid;

use crate::routes::user::LoginCredentials;

#[derive(Clone)]
pub struct Backend {
    db: PgPool,
}

impl Backend {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    TaskJoin(#[from] task::JoinError),
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    user_id: Uuid,
    password_hash: String,
}

impl AuthenticatedUser {
    pub fn new(user_id: Uuid, password_hash: String) -> Self {
        Self {
            user_id,
            password_hash,
        }
    }
}

impl AuthUser for AuthenticatedUser {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.user_id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = AuthenticatedUser;

    type Credentials = LoginCredentials;

    type Error = AuthError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user: Option<Self::User> = sqlx::query_as!(
            AuthenticatedUser,
            "select user_id,password_hash from users where username = $1",
            creds.username
        )
        .fetch_optional(&self.db)
        .await?;

        task::spawn_blocking(|| {
            Ok(user.filter(|user| verify_password(creds.password, &user.password_hash).is_ok()))
        })
        .await?
    }

    async fn get_user(
        &self,
        user_id: &axum_login::UserId<Self>,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as!(
            AuthenticatedUser,
            "select user_id,password_hash from users where user_id = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(user)
    }
}

// We use a type alias for convenience.
//
// Note that we've supplied our concrete backend here.
pub type AuthSession = axum_login::AuthSession<Backend>;

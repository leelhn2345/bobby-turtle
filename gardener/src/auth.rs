use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend};
use password_auth::verify_password;
use serde::{Deserialize, Serialize};
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

#[derive(PartialEq, sqlx::Type, Serialize, Deserialize, Clone, Debug, Eq, PartialOrd, Ord)]
#[sqlx(type_name = "permissions", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PermissionLevel {
    Member,
    Admin,
    Alpha,
}

impl PermissionLevel {
    pub fn member() -> Self {
        PermissionLevel::Member
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub permission_level: PermissionLevel,
    pub verified: bool,
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
            r#"
            select 
            user_id, 
            username, 
            password_hash, 
            permission_level as "permission_level: PermissionLevel",
            verified
            from users where username = $1
            "#,
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
            r#"
            select 
            user_id, 
            username, 
            password_hash,
            permission_level as "permission_level:PermissionLevel",
            verified
            from users where user_id = $1
            "#,
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

use std::time::Duration;

use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    PgPool,
};

use crate::environment::Environment;

#[derive(Deserialize, Debug, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    #[must_use]
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    #[must_use]
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }
}

#[must_use]
pub async fn get_connection_pool(env: &Environment, db_settings: &DatabaseSettings) -> PgPool {
    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(db_settings.with_db());

    if *env == Environment::Production {
        sqlx::migrate!("../migrations")
            .run(&pool)
            .await
            .expect("cannot run db migration");
    }

    pool
}

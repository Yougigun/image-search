use anyhow::{Context, Result};
use derive_more::{Deref, From, Into};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions, Pool, Postgres,
};

#[derive(Deref, From, Into, Clone)]
pub struct PostgresClient(Pool<Postgres>);

#[derive(Default)]
pub struct PostgresClientConfig {
    pub hostname: String,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub db_name: String,
}

impl PostgresClient {
    pub async fn build(config: &PostgresClientConfig) -> Result<Self> {
        let url = config.build_url();

        let client = PgPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .context(format!("failed to connect to database: {}", url))?;

        tracing::info!("postgres client connected successfully on {}", url);

        Ok(Self(client))
    }

    pub fn into_inner(self) -> Pool<Postgres> {
        self.0
    }
}

impl PostgresClientConfig {
    // [See this](https://www.postgresql.org/docs/current/libpq-connect.html#LIBPQ-CONNSTRING)
    fn build_url(&self) -> String {
        let mut options = PgConnectOptions::new()
            .host(&self.hostname)
            .database(&self.db_name);

        if let Some(port) = self.port {
            options = options.port(port);
        }

        if let Some(user) = &self.user {
            options = options.username(user);
        }

        if let Some(password) = &self.password {
            options = options.password(password);
        }
        

        options.to_url_lossy().to_string()
    }
}

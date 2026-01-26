use sqlx::PgPool;

use crate::domain::CoreError;

pub struct Postgres {
    pool: PgPool,
}

pub struct PostgresConfig {
    pub database_url: String,
}

impl Postgres {
    pub async fn new(config: PostgresConfig) -> Result<Self, CoreError> {
        let database_url = config.database_url.clone();

        let pool =
            PgPool::connect(&database_url)
                .await
                .map_err(|_| CoreError::ServiceUnavailable {
                    service: "Postgres".into(),
                })?;

        Ok(Self { pool })
    }

    pub fn get_db(&self) -> PgPool {
        self.pool.clone()
    }
}

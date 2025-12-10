use std::sync::Arc;

use beep_auth::infrastructure::keycloak_repository::KeycloakAuthRepository;

use crate::{
    application::services::ApplicationService,
    domain::{Config, CoreError, services::{broker::BrokerServiceImpl, notification::NotificationServiceImpl}},
    infrastructure::{
        db::postgres::{Postgres, PostgresConfig},
        repositories::notification::PostgresNotificationRepository,
    },
};

pub mod http;
pub mod services;

pub async fn create_service(config: Config) -> Result<ApplicationService, CoreError> {
    let postgres: Postgres = Postgres::new(PostgresConfig {
        database_url: config.database.database_url.clone(),
    })
    .await?;
    let auth_repository = KeycloakAuthRepository::new(&config.auth.issuer, None);
    let notification_repository = PostgresNotificationRepository::new(postgres.get_db());

    let app = ApplicationService {
        auth_repository: auth_repository,
        notification_service: NotificationServiceImpl::new(notification_repository.clone()),
        broker_service: BrokerServiceImpl::new(notification_repository, Arc::new(config.broker)).await?,
    };

    Ok(app)
}

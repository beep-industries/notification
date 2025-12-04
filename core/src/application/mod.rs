use beep_auth::{
    domain::ports::HasAuthRepository, infrastructure::keycloak_repository::KeycloakAuthRepository,
};

use crate::{domain::{Config, CoreError, services::{HasNotificationRepository, Service}}, infrastructure::{db::postgres::{Postgres, PostgresConfig}, repositories::notification::PostgresNotificationRepository}};

pub type BeepService = Service<KeycloakAuthRepository, PostgresNotificationRepository>;

impl HasAuthRepository for BeepService {
    type AuthRepo = KeycloakAuthRepository;

    fn auth_repository(&self) -> &Self::AuthRepo {
        &self.auth_repository
    }
}

impl HasNotificationRepository for BeepService {
    type NotificationRepo = PostgresNotificationRepository;

    fn notification_repository(&self) -> &Self::NotificationRepo {
        &self.notification_repository
    }
}

pub async fn create_service(config: Config) -> Result<BeepService, CoreError> {
    let postgres = Postgres::new(PostgresConfig { 
        database_url: config.database.database_url.clone(),
    }).await?;
    let auth_repository = KeycloakAuthRepository::new(&config.auth.issuer, None);
    let notification_repository = PostgresNotificationRepository::new(postgres.get_db());

    Ok(BeepService { auth_repository, notification_repository })
}

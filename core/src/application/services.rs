use beep_auth::{domain::ports::HasAuthRepository, infrastructure::keycloak_repository::KeycloakAuthRepository};

use crate::{domain::notification::service::NotificationService, infrastructure::repositories::notification::PostgresNotificationRepository};

type AuthRepo = KeycloakAuthRepository;
type NotifRepo = PostgresNotificationRepository;


impl HasAuthRepository for ApplicationService {
    type AuthRepo = KeycloakAuthRepository;

    fn auth_repository(&self) -> &Self::AuthRepo {
        &self.auth_repository
    }
}

#[derive(Clone)]
pub struct ApplicationService {
    pub auth_repository: AuthRepo,
    pub notification_service: NotificationService<NotifRepo>,
}
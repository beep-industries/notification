use beep_auth::{domain::{models::Identity, ports::HasAuthRepository}, infrastructure::keycloak_repository::KeycloakAuthRepository};

use crate::{domain::{CoreError, entities::notification::Notification, notification::service::NotificationService}, infrastructure::repositories::notification::PostgresNotificationRepository};

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

// impl notification_service for ApplicationService {} // TODO 
pub async fn get_notifications_for_user(
    service: &ApplicationService,
    identity: Identity,
    user_id: &str,
) -> Result<Vec<Notification>, CoreError>{
    service.notification_service.get_notifications_for_user(identity, user_id).await
}
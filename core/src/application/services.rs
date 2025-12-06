use beep_auth::{
    domain::{models::Identity, ports::HasAuthRepository},
    infrastructure::keycloak_repository::KeycloakAuthRepository,
};

use crate::{
    domain::{
        CoreError,
        entities::{notification::Notification, preference::NotificationPreference},
        notification::service::NotificationServiceImpl,
        ports::notification::NotificationService,
    },
    infrastructure::repositories::notification::PostgresNotificationRepository,
};

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
    pub notification_service: NotificationServiceImpl<NotifRepo>,
}

impl NotificationService for ApplicationService {
    fn get_notifications_for_user(
        &self,
        identity: Identity,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<Notification>, CoreError>> + Send {
        self.notification_service
            .get_notifications_for_user(identity, user_id)
    }
    fn mark_notification_as_read(
        &self,
        identity: Identity,
        user_id: String,
        notification_id: String,
    ) -> impl Future<Output = Result<(), CoreError>> + Send {
        self.notification_service
            .mark_notification_as_read(identity, user_id, notification_id)
    }

    fn get_preferences(
        &self,
        identity: Identity,
        user_id: String,
    ) -> impl Future<Output = Result<Vec<NotificationPreference>, CoreError>> + Send {
        self.notification_service.get_preferences(identity, user_id)
    }

    fn update_preferences(
        &self,
        identity: Identity,
        user_id: String,
        notification_preferences: NotificationPreference,
    ) -> impl Future<Output = Result<(), CoreError>> + Send {
        self.notification_service
            .update_preferences(identity, user_id, notification_preferences)
    }
}

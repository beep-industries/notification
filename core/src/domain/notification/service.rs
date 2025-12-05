use beep_auth::domain::models::Identity;
use uuid::Uuid;

use crate::domain::{
    CoreError,
    entities::{UserId, notification::Notification},
    ports::notification::NotificationRepository,
};

#[derive(Clone)]
pub struct NotificationService<N>
where
    N: NotificationRepository,
{
    pub notification_repository: N,
}

impl<N> NotificationService<N>
where
    N: NotificationRepository,
{
    pub fn new(notification_repository: N) -> Self {
        NotificationService {
            notification_repository,
        }
    }

    pub async fn get_notifications_for_user(
        &self,
        identity: Identity,
        user_id: &str,
    ) -> Result<Vec<Notification>, CoreError> {
        if identity.id() != user_id {
            return Err(CoreError::Unauthorized);
        }
        self.notification_repository
            .get_notifications_for_user(UserId(Uuid::parse_str(user_id).map_err(|_| {
                CoreError::FailedGetNotification {
                    message: "Invalid user ID format".to_string(),
                }
            })?))
            .await
    }
}

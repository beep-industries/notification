use crate::domain::{
    CoreError,
    entities::{NotificationId, UserId, notification::{InsertNotificationInput, Notification}},
};

pub trait NotificationRepository: Send + Sync {
    fn insert(
        &self,
        input: InsertNotificationInput,
    ) -> impl Future<Output = Result<Notification, CoreError>> + Send;

    fn get_notifications_for_user(
        &self,
        user_id: UserId,
    ) -> impl Future<Output = Result<Vec<Notification>, CoreError>> + Send;

    fn mark_notification_as_read(
        &self,
        user_id: UserId,
        notification_id: NotificationId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

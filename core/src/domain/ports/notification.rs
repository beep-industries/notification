use beep_auth::domain::models::Identity;

use crate::domain::{
    CoreError,
    entities::{
        NotificationId, UserId,
        notification::{InsertNotificationInput, Notification, UpdateNotificationInput},
        preference::NotificationPreference,
    },
};

pub trait NotificationService: Send + Sync {
    fn get_notifications_for_user(
        &self,
        identity: Identity,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<Notification>, CoreError>> + Send;

    fn mark_notification_as_read(
        &self,
        identity: Identity,
        user_id: String,
        notification_id: String,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn get_preferences(
        &self,
        identity: Identity,
        user_id: String,
    ) -> impl Future<Output = Result<Vec<NotificationPreference>, CoreError>> + Send;

    fn update_preferences(
        &self,
        identity: Identity,
        user_id: String,
        notification_preferences: NotificationPreference,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait NotificationRepository: Send + Sync {
    fn insert(
        &self,
        input: InsertNotificationInput,
    ) -> impl Future<Output = Result<Notification, CoreError>> + Send;

    fn insert_message_notification(
        &self,
        input: InsertNotificationInput,
    ) -> impl Future<Output = Result<Notification, CoreError>> + Send;

    fn update_message_notification(
        &self,
        input: UpdateNotificationInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn update(
        &self,
        notification: UpdateNotificationInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn delete_message_notification(
        &self,
        message_id: NotificationId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn delete_friend_request_notification(
        &self,
        friend_request_id: NotificationId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn delete(
        &self,
        notification_id: NotificationId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn get_notifications_for_user(
        &self,
        user_id: UserId,
    ) -> impl Future<Output = Result<Vec<Notification>, CoreError>> + Send;

    fn mark_notification_as_read(
        &self,
        user_id: UserId,
        notification_id: NotificationId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn get_preferences(
        &self,
        user_id: UserId,
    ) -> impl Future<Output = Result<Vec<NotificationPreference>, CoreError>> + Send;

    fn update_notification_preferences(
        &self,
        user_id: UserId,
        notification_preferences: NotificationPreference,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

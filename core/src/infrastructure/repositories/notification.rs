use std::fmt::Debug;

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    CoreError,
    entities::{
        NotificationId, UserId,
        notification::{InsertNotificationInput, Notification, UpdateNotificationInput},
        preference::NotificationPreference,
    },
    ports::notification::NotificationRepository,
};

#[derive(Clone)]
pub struct PostgresNotificationRepository {
    pool: PgPool,
}

impl PostgresNotificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl NotificationRepository for PostgresNotificationRepository {
    #[allow(unused)]
    async fn insert(&self, input: InsertNotificationInput) -> Result<Notification, CoreError> {
        unimplemented!()
    }

    async fn insert_message_notification(
        &self,
        input: InsertNotificationInput,
    ) -> Result<Notification, CoreError> {
        let notification: Notification = input.into();
        let message_id: Uuid;
        match notification.message_id {
            None => {
                return Err(CoreError::FailedInsertNotification {
                    message: "message_id must be provided for message notifications".to_string(),
                });
            }
            Some(id) => message_id = id.into(),
        }

        let user_id: Uuid = notification.user_id.into();
        let channel_id: Uuid = notification.channel_id.into();
        let notification_id: Uuid = notification.id.into();

        sqlx::query!(
            r#"
            INSERT INTO notifications (id, message_id, channel_id, user_id, title, message, notification_type, status, created_at, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            notification_id,
            message_id,
            channel_id,
            user_id,
            notification.title,
            notification.message,
            notification.notification_type.to_string(),
            notification.status.to_string(),
            notification.created_at,
            notification.metadata,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::FailedInsertNotification {
            message: e.to_string(),
        })?;

        Ok(notification)
    }

    async fn update_message_notification(
        &self,
        input: UpdateNotificationInput,
    ) -> Result<(), CoreError> {
        let message_id: Uuid;
        match input.message_id {
            None => {
                return Err(CoreError::FailedUpdateNotification {
                    message: "message_id must be provided for message notifications".to_string(),
                });
            }
            Some(id) => message_id = id.into(),
        }

        let result = sqlx::query!(
            r#"
        UPDATE notifications
        SET 
            message = COALESCE($1, message),
            metadata = metadata || $2::jsonb
        WHERE message_id = $3
        "#,
            input.message,
            input.metadata,
            message_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::FailedUpdateNotification {
            message: e.to_string(),
        })?;

        if result.rows_affected() == 0 {
            return Err(CoreError::FailedUpdateNotification {
                message: "notification not found".to_string(),
            });
        }
        Ok(())
    }

    #[allow(unused)]
    async fn update(&self, input: UpdateNotificationInput) -> Result<(), CoreError> {
        unimplemented!()
    }

    async fn delete_message_notification(
        &self,
        message_id: NotificationId,
    ) -> Result<(), CoreError> {
        let message_id: Uuid = message_id.into();
        let result = sqlx::query!(
            r#"
            DELETE FROM notifications
            WHERE message_id = $1
            "#,
            message_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::FailedDeleteNotification {
            message: e.to_string(),
        })?;

        if result.rows_affected() == 0 {
            return Err(CoreError::FailedDeleteNotification {
                message: "notification not found".to_string(),
            });
        }

        Ok(())
    }

    async fn delete_friend_request_notification(
        &self,
        _friend_request_id: NotificationId,
    ) -> Result<(), CoreError> {
        unimplemented!("Friend request notification deletion not yet implemented")
    }

    #[allow(unused)]
    async fn delete(
        &self,
        notification_id: crate::domain::entities::NotificationId,
    ) -> Result<(), CoreError> {
        unimplemented!()
    }

    async fn get_notifications_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<Notification>, CoreError> {
        let user_id: Uuid = user_id.into();

        let records = sqlx::query!(
            r#"
            SELECT id, message_id, friend_request_id, channel_id, user_id, title, message, notification_type, status, created_at, metadata, sent_at
            FROM notifications
            WHERE user_id = $1 and status = 'Sent'
            "#,
            user_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::FailedGetNotification {
            message: e.to_string(),
        })?;

        let notifications = records
            .into_iter()
            .map(|record| {
                let message_id: Option<NotificationId> = match record.message_id {
                    None => None,
                    Some(id) => Some(id.into()),
                };

                let friend_request_id: Option<NotificationId> = match record.friend_request_id {
                    None => None,
                    Some(id) => Some(id.into()),
                };
                Notification {
                    id: record.id.into(),
                    message_id: message_id,
                    friend_request_id: friend_request_id,
                    channel_id: record.channel_id.into(),
                    user_id: record.user_id.into(),
                    title: record.title,
                    message: record.message,
                    notification_type: record.notification_type.into(),
                    status: record.status.into(),
                    created_at: record.created_at,
                    metadata: record.metadata,
                    sent_at: record.sent_at,
                    read_at: None,
                }
            })
            .collect();

        Ok(notifications)
    }

    async fn mark_notification_as_read(
        &self,
        user_id: UserId,
        notification_id: crate::domain::entities::NotificationId,
    ) -> Result<(), CoreError> {
        let user_id: Uuid = user_id.into();
        let notification_id: Uuid = notification_id.into();

        let result = sqlx::query!(
            r#"
            UPDATE notifications
            SET status = 'Read', read_at = NOW()
            WHERE id = $1 AND user_id = $2
            "#,
            notification_id,
            user_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::FailedMarkNotificationAsRead {
            message: e.to_string(),
        })?;

        if result.rows_affected() == 0 {
            return Err(CoreError::FailedMarkNotificationAsRead {
                message: "notification not found".to_string(),
            });
        }

        Ok(())
    }

    async fn get_preferences(
        &self,
        user_id: UserId,
    ) -> Result<Vec<NotificationPreference>, CoreError> {
        let user_id: Uuid = user_id.into();

        let records = sqlx::query!(
            r#"
            SELECT *
            FROM notification_preferences
            WHERE user_id = $1
            "#,
            user_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::FailedGetPreferences {
            message: e.to_string(),
        })?;

        let preferences = records
            .into_iter()
            .map(|record| NotificationPreference {
                id: record.id.into(),
                user_id: record.user_id.into(),
                channel_id: record.channel_id.into(),
                enabled: record.enabled,
                muted_until: record.muted_until,
            })
            .collect();

        Ok(preferences)
    }

    async fn update_notification_preferences(
        &self,
        user_id: UserId,
        notification_preferences: NotificationPreference,
    ) -> Result<(), CoreError> {
        let user_id: Uuid = user_id.into();

        let channel_id: Uuid = notification_preferences.channel_id.into();
        let result = sqlx::query!(
            r#"
            UPDATE notification_preferences
            SET enabled = $1, muted_until = $2
            WHERE user_id = $3 AND channel_id = $4
            "#,
            notification_preferences.enabled,
            notification_preferences.muted_until,
            user_id,
            channel_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::FailedUpdatePreferences {
            message: e.to_string(),
        })?;

        if result.rows_affected() == 0 {
            return Err(CoreError::PreferenceNotFound);
        }

        Ok(())
    }
}

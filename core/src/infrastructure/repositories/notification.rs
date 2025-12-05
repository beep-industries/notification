use std::fmt::Debug;

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    CoreError,
    entities::{
        UserId,
        notification::{InsertNotificationInput, Notification},
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
    async fn insert(&self, input: InsertNotificationInput) -> Result<Notification, CoreError> {
        let notification: Notification = input.into();

        let user_id: Uuid = notification.user_id.into();
        let channel_id: Uuid = notification.channel_id.into();
        let notification_id: Uuid = notification.id.into();

        sqlx::query!(
            r#"
            INSERT INTO notifications (id, channel_id, user_id, title, message, notification_type, status, created_at, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            notification_id,
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

    async fn get_notifications_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<Notification>, CoreError> {
        let user_id: Uuid = user_id.into();

        let records = sqlx::query!(
            r#"
            SELECT id, channel_id, user_id, title, message, notification_type, status, created_at, metadata, sent_at
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
            .map(|record| Notification {
                id: record.id.into(),
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
}

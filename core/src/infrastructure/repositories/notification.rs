use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    CoreError,
    entities::notification::{InsertNotificationInput, Notification},
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
}

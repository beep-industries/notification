use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::{
    entities::{ChannelId, NotificationId, UserId},
    services::generate_id,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed,
    Read,
}

impl std::fmt::Display for NotificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            NotificationStatus::Pending => "pending",
            NotificationStatus::Sent => "sent",
            NotificationStatus::Failed => "failed",
            NotificationStatus::Read => "read",
        };
        write!(f, "{}", s)
    }
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            NotificationType::Info => "info",
            NotificationType::Warning => "warning",
            NotificationType::Error => "error",
            NotificationType::Success => "success",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: NotificationId,
    pub user_id: UserId,
    pub channel_id: ChannelId,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub status: NotificationStatus,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

impl From<InsertNotificationInput> for Notification {
    fn from(value: InsertNotificationInput) -> Self {
        let now = Utc::now();
        Self {
            id: NotificationId(generate_id()),
            channel_id: value.channel_id,
            user_id: value.user_id,
            title: value.title,
            message: value.message,
            metadata: value.metadata,
            notification_type: value.notification_type,
            status: NotificationStatus::Pending,
            created_at: now,
            read_at: None,
            sent_at: None,
        }
    }
}

pub struct InsertNotificationInput {
    pub user_id: UserId,
    pub channel_id: ChannelId,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub metadata: Option<serde_json::Value>,
}

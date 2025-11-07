use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::entities::{ChannelId, NotificationId, UserId};

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


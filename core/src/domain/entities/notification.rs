use crate::domain::{
    entities::{ChannelId, NotificationId, UserId},
    generate_id,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationType {
    Friend,
    Message,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

impl From<String> for NotificationStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "pending" => NotificationStatus::Pending,
            "sent" => NotificationStatus::Sent,
            "failed" => NotificationStatus::Failed,
            "read" => NotificationStatus::Read,
            _ => NotificationStatus::Failed,
        }
    }
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            NotificationType::Friend => "friend",
            NotificationType::Message => "message",
        };
        write!(f, "{}", s)
    }
}

impl From<String> for NotificationType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "friend" => NotificationType::Friend,
            "message" => NotificationType::Message,
            _ => NotificationType::Message,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Notification {
    pub id: NotificationId,
    pub message_id: Option<NotificationId>,
    pub friend_request_id: Option<NotificationId>,
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
            message_id: value.message_id,
            friend_request_id: value.friend_request_id,
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
    pub message_id: Option<NotificationId>,
    pub friend_request_id: Option<NotificationId>,
    pub user_id: UserId,
    pub channel_id: ChannelId,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub metadata: Option<serde_json::Value>,
}

pub struct UpdateNotificationInput {
    pub message_id: Option<NotificationId>,
    pub friend_request_id: Option<NotificationId>,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
}

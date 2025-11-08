use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod events;
pub mod notification;
pub mod preference;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChannelId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NotificationId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PreferenceId(pub Uuid);

impl std::fmt::Display for PreferenceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for PreferenceId {
    fn from(uuid: Uuid) -> Self {
        PreferenceId(uuid)
    }
}

impl From<Uuid> for NotificationId {
    fn from(uuid: Uuid) -> Self {
        NotificationId(uuid)
    }
}

impl From<Uuid> for ChannelId {
    fn from(uuid: Uuid) -> Self {
        ChannelId(uuid)
    }
}

impl From<Uuid> for UserId {
    fn from(uuid: Uuid) -> Self {
        UserId(uuid)
    }
}

impl From<UserId> for Uuid {
    fn from(user_id: UserId) -> Self {
        user_id.0
    }
}

impl From<ChannelId> for Uuid {
    fn from(channel_id: ChannelId) -> Self {
        channel_id.0
    }
}

impl From<NotificationId> for Uuid {
    fn from(notification_id: NotificationId) -> Self {
        notification_id.0
    }
}

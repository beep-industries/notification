use chrono::Utc;
use uuid::{NoContext, Timestamp, Uuid};

pub mod events;
pub mod preference;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(unused)]
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

use uuid::Uuid;

pub mod events;
pub mod preference;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(unused)]
pub struct NotificationId(pub Uuid);

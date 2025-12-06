use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::domain::{
    entities::{ChannelId, NotificationId, UserId},
    generate_id,
};

#[derive(Debug, Clone, Serialize, PartialEq, Validate, Deserialize)]
#[allow(unused)]
pub struct NotificationPreference {
    pub id: NotificationId,
    pub user_id: UserId,
    pub channel_id: ChannelId,
    pub enabled: bool,
    pub muted_until: Option<DateTime<Utc>>,
}

#[allow(unused)]
impl NotificationPreference {
    pub fn new(user_id: UserId, channel_id: ChannelId) -> Self {
        let id = generate_id();
        Self {
            id: id.into(),
            user_id,
            channel_id,
            enabled: true,
            muted_until: None,
        }
    }

    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn mute_until(mut self, until: DateTime<Utc>) -> Self {
        self.muted_until = Some(until);
        self
    }

    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        if !self.enabled {
            return false;
        }
        if let Some(until) = self.muted_until {
            if now < until {
                return false;
            }
        }
        true
    }
}

use chrono::{DateTime, Utc};

use crate::domain::entities::{ChannelId, UserId};

#[derive(Debug, Clone)]
pub struct NotificationPreference {
    pub user_id: UserId,
    pub channel_id: ChannelId,
    pub enabled: bool,
    pub muted_until: Option<DateTime<Utc>>,
}

impl NotificationPreference {
    pub fn new(user_id: UserId, channel_id: ChannelId) -> Self {
        Self {
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

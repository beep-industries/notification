use chrono::{DateTime, Utc};

use crate::domain::entities::{SessionId, UserId};

#[derive(Debug, Clone)]
pub struct WebSocketSession {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub connected_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

impl WebSocketSession {
    pub fn new(user_id: UserId, session_id: SessionId) -> Self {
        let now = Utc::now();
        Self {
            user_id,
            session_id,
            connected_at: now,
            last_seen: now,
        }
    }

    pub fn touch(&mut self, now: DateTime<Utc>) {
        self.last_seen = now;
    }
}

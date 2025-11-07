use beep_auth::domain::ports::AuthRepository;
use chrono::Utc;
use uuid::{NoContext, Timestamp, Uuid};

pub fn generate_id() -> Uuid {
    let now = Utc::now();
    let seconds = now.timestamp().try_into().unwrap_or(0);
    let timestamp = Timestamp::from_unix(NoContext, seconds, 0);
    Uuid::new_v7(timestamp)
}

#[derive(Clone)]
pub struct Service<A>
where
    A: AuthRepository,
{
    pub(crate) auth_repository: A,
}

// use beep_auth::domain::ports::AuthRepository;

use chrono::Utc;
use uuid::{NoContext, Timestamp, Uuid};

// use crate::domain::ports::notification::NotificationRepository;

// pub trait HasNotificationRepository {
//     type NotificationRepo: NotificationRepository;

//     fn notification_repository(&self) -> &Self::NotificationRepo;
// }

pub fn generate_id() -> Uuid {
    let now = Utc::now();
    let seconds = now.timestamp().try_into().unwrap_or(0);
    let timestamp = Timestamp::from_unix(NoContext, seconds, 0);
    Uuid::new_v7(timestamp)
}

// #[derive(Clone)]
// pub struct Service<A, N>
// where
//     A: AuthRepository,
//     N: NotificationRepository,
// {
//     pub(crate) auth_repository: A,
//     pub(crate) notification_repository: N,
// }

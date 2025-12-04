use crate::domain::ports::notification::NotificationRepository;

#[derive(Clone)]
pub struct NotificationService<N> where N: NotificationRepository {
    pub notification_repository: N,
}

impl<N> NotificationService<N> where N: NotificationRepository {
    pub fn new(notification_repository: N) -> Self {
        NotificationService {
            notification_repository,
        }
    }
}
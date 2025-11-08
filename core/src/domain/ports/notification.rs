use crate::domain::{
    CoreError,
    entities::notification::{InsertNotificationInput, Notification},
};

pub trait NotificationRepository: Send + Sync {
    fn insert(
        &self,
        input: InsertNotificationInput,
    ) -> impl Future<Output = Result<Notification, CoreError>> + Send;
}

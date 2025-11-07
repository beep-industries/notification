use crate::domain::{
    CoreError,
    entities::{PreferenceId, preference::NotificationPreference},
};

pub trait PreferenceRepository: Send + Sync {
    fn find_by_id(
        &self,
        id: &PreferenceId,
    ) -> impl Future<Output = Result<Option<NotificationPreference>, CoreError>> + Send;
    fn insert(&self) -> impl Future<Output = Result<NotificationPreference, CoreError>> + Send;
}

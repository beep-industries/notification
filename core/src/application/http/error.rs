use beep_server::ApiError;

use crate::domain::CoreError;

impl From<CoreError> for ApiError {
    fn from(error: CoreError) -> Self {
        match error {
            // TODO modify unknown to specific error types
            CoreError::FailedGetNotification { message } => Self::Unknown { message },
            CoreError::FailedInsertNotification { message } => Self::Unknown { message },
            CoreError::ServiceUnavailable { service } => Self::Unknown { message: service },
            CoreError::PreferenceNotFound { id } => Self::Unknown {
                message: id.to_string(),
            },
            CoreError::FailedMarkNotificationAsRead { message } => Self::Unknown { message },
            CoreError::FailedGetPreferences { message } => Self::Unknown { message },
            CoreError::FailedUpdatePreferences { message } => Self::Unknown { message },
            CoreError::Unauthorized => Self::Unknown {
                message: "unauthorized access".to_string(),
            },
        }
    }
}

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use beep_server::ApiErrorResponse;
use thiserror::Error;
use tracing::error;

use crate::domain::CoreError;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    Server(#[from] beep_server::ApiError),
    #[error("Notification not found")]
    NotificationNotFound,
    #[error("Failed to insert notification")]
    NotificationInsertionFailed,
    #[error("Preference not found")]
    PreferenceNotFound,
    #[error("Failed to update preferences")]
    PreferenceUpdateFailed,
    #[error("Failed to mark notification as read")]
    MarkAsReadFailed,
    #[error("Bad request: {message}")]
    BadRequest { message: String },
    #[error("Service unavailable")]
    ServiceUnavailable,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Validation error: {message}")]
    ValidationError { message: String },
}

impl From<CoreError> for ApiError {
    fn from(error: CoreError) -> Self {
        match error {
            CoreError::FailedGetNotification { message } => {
                error!("Failed to get notification: {}", message);
                Self::NotificationNotFound
            }
            CoreError::FailedInsertNotification { message } => {
                error!("Failed to insert notification: {}", message);
                Self::NotificationInsertionFailed
            }
            CoreError::ServiceUnavailable { service } => {
                error!("Service unavailable: {}", service);
                Self::ServiceUnavailable
            }
            CoreError::PreferenceNotFound => {
                error!("Preference not found");
                Self::PreferenceNotFound
            }
            CoreError::FailedMarkNotificationAsRead { message } => {
                error!("Failed to mark notification as read: {}", message);
                Self::MarkAsReadFailed
            }
            CoreError::FailedGetPreferences { message } => {
                error!("Failed to get preferences: {}", message);
                Self::PreferenceNotFound
            }
            CoreError::FailedUpdatePreferences { message } => {
                error!("Failed to update preferences: {}", message);
                Self::PreferenceUpdateFailed
            }
            CoreError::Unauthorized => {
                error!("Unauthorized access");
                Self::Unauthorized
            }
            _ => {
                error!("Unhandled core error: {:?}", error);
                Self::ServiceUnavailable
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Server(err) => err.into_response(),
            ApiError::NotificationNotFound => (
                StatusCode::NOT_FOUND,
                Json(ApiErrorResponse {
                    code: "E_NOTIFICATION_NOT_FOUND".to_string(),
                    status: StatusCode::NOT_FOUND.as_u16(),
                    message: "Notification not found".to_string(),
                }),
            )
                .into_response(),
            ApiError::NotificationInsertionFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    code: "E_NOTIFICATION_INSERTION_FAILED".to_string(),
                    status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    message: "Failed to insert notification".to_string(),
                }),
            )
                .into_response(),
            ApiError::PreferenceNotFound => (
                StatusCode::NOT_FOUND,
                Json(ApiErrorResponse {
                    code: "E_PREFERENCE_NOT_FOUND".to_string(),
                    status: StatusCode::NOT_FOUND.as_u16(),
                    message: "Preference not found".to_string(),
                }),
            )
                .into_response(),
            ApiError::PreferenceUpdateFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    code: "E_PREFERENCE_UPDATE_FAILED".to_string(),
                    status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    message: "Failed to update preferences".to_string(),
                }),
            )
                .into_response(),
            ApiError::MarkAsReadFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    code: "E_MARK_AS_READ_FAILED".to_string(),
                    status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    message: "Failed to mark notification as read".to_string(),
                }),
            )
                .into_response(),
            ApiError::BadRequest { message } => (
                StatusCode::BAD_REQUEST,
                Json(ApiErrorResponse {
                    code: "E_BAD_REQUEST".to_string(),
                    status: StatusCode::BAD_REQUEST.as_u16(),
                    message,
                }),
            )
                .into_response(),
            ApiError::ServiceUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiErrorResponse {
                    code: "E_SERVICE_UNAVAILABLE".to_string(),
                    status: StatusCode::SERVICE_UNAVAILABLE.as_u16(),
                    message: "Service unavailable".to_string(),
                }),
            )
                .into_response(),
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(ApiErrorResponse {
                    code: "E_UNAUTHORIZED".to_string(),
                    status: StatusCode::UNAUTHORIZED.as_u16(),
                    message: "Unauthorized".to_string(),
                }),
            )
                .into_response(),
            ApiError::ValidationError { message } => (
                StatusCode::BAD_REQUEST,
                Json(ApiErrorResponse {
                    code: "E_VALIDATION_ERROR".to_string(),
                    status: StatusCode::BAD_REQUEST.as_u16(),
                    message,
                }),
            )
                .into_response(),
        }
    }
}

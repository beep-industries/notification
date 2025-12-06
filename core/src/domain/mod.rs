use beep_server::config::AuthConfig;
use thiserror::Error;

pub mod entities;
pub mod notification;
pub mod ports;
pub mod services;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: String },

    #[error("Preference not found")]
    PreferenceNotFound,

    #[error("Insert notification failed: {message}")]
    FailedInsertNotification { message: String },

    #[error("Get notification failed: {message}")]
    FailedGetNotification { message: String },

    #[error("Mark notification as read failed: {message}")]
    FailedMarkNotificationAsRead { message: String },

    #[error("Get preferences failed: {message}")]
    FailedGetPreferences { message: String },

    #[error("Update notification preferences failed: {message}")]
    FailedUpdatePreferences { message: String },

    #[error("Unauthorized")]
    Unauthorized,
}

pub struct Config {
    pub auth: AuthConfig,
    pub database: DatabaseConfig,
}

pub struct DatabaseConfig {
    pub database_url: String,
}

use beep_server::config::AuthConfig;
use thiserror::Error;

pub mod entities;
pub mod ports;
pub mod services;

use chrono::Utc;
use uuid::{NoContext, Timestamp, Uuid};

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

pub struct QueueBinding {
    pub queue_name: String,
    pub exchange_name: String,
}

pub struct RabbitmqConfig {
    pub rabbitmq_url: String,
    pub rabbitmq_bindings: Vec<QueueBinding>,
}

pub fn generate_id() -> Uuid {
    let now = Utc::now();
    let seconds = now.timestamp().try_into().unwrap_or(0);
    let timestamp = Timestamp::from_unix(NoContext, seconds, 0);
    Uuid::new_v7(timestamp)
}

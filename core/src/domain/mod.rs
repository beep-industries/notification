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

    #[error("Update notification failed: {message}")]
    FailedUpdateNotification { message: String },

    #[error("Delete notification failed: {message}")]
    FailedDeleteNotification { message: String },

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

    #[error("Failed to create channel: {message}")]
    FailedCreateChannel { message: String },

    #[error("Create exchange failed: {message}")]
    FailedCreateExchange { message: String },

    #[error("Create queue failed: {message}")]
    FailedCreateQueue { message: String },

    #[error("Bind queue failed: {message}")]
    FailedBindQueue { message: String },

    #[error("Internal error: {service}")]
    InternalError { service: String },

    #[error("Ack error: {message}")]
    AckError { message: String },

    #[error("Unsupported handler: {handler}")]
    UnsupportedHandler { handler: String },

    #[error("Deserialize error: {message}")]
    DeserializeError { message: String },

    #[error("Failed to get create consumer: {message}")]
    FailedCreateConsumer { message: String },
}

pub struct Config {
    pub auth: AuthConfig,
    pub database: DatabaseConfig,
    pub broker: BrokerConfig,
}

pub struct DatabaseConfig {
    pub database_url: String,
}

#[derive(Debug, Clone)]
pub struct QueueBinding {
    pub queue_name: String,
    pub exchange_name: String,
}

pub struct BrokerConfig {
    pub broker_url: String,
    pub broker_bindings: Vec<QueueBinding>,
}

pub fn generate_id() -> Uuid {
    let now = Utc::now();
    let seconds = now.timestamp().try_into().unwrap_or(0);
    let timestamp = Timestamp::from_unix(NoContext, seconds, 0);
    Uuid::new_v7(timestamp)
}

use beep_server::config::AuthConfig;
use thiserror::Error;

use crate::domain::entities::PreferenceId;

pub mod entities;
pub mod ports;
pub mod services;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: String },

    #[error("Preference with id {id} not found")]
    PreferenceNotFound { id: PreferenceId },

    #[error("Insert notification failed: {message}")]
    FailedInsertNotification { message: String },
}

pub struct Config {
    pub auth: AuthConfig,
    pub database: DatabaseConfig,
}

pub struct DatabaseConfig {
    pub database_url: String,
}
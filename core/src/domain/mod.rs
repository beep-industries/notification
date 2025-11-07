use thiserror::Error;

use crate::domain::entities::PreferenceId;

pub mod entities;
pub mod ports;
pub mod services;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Preference with id {id} not found")]
    PreferenceNotFound { id: PreferenceId },
}

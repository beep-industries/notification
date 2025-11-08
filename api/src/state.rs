use core::{
    application::{BeepService, create_service},
    domain::Config,
};
use std::sync::Arc;

use beep_server::ApiError;

use crate::args::Args;

#[derive(Clone)]
pub struct AppState {
    pub args: Arc<Args>,
    pub service: BeepService,
}

pub async fn state(args: Arc<Args>) -> Result<AppState, ApiError> {
    let config: Config = args.as_ref().clone().into();

    let service = create_service(config)
        .await
        .map_err(|e| ApiError::Unknown {
            message: e.to_string(),
        })?;

    Ok(AppState { args, service })
}

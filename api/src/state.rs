use core::{
    application::{create_service, services::ApplicationService},
    domain::{Config, CoreError},
};
use std::sync::Arc;

use crate::args::Args;

#[derive(Clone)]
pub struct AppState {
    pub args: Arc<Args>,
    pub service: ApplicationService,
}

pub async fn state(args: Arc<Args>) -> Result<AppState, CoreError> {
    let config: Config = args.as_ref().clone().into();

    let service = create_service(config).await?;

    Ok(AppState { args, service })
}

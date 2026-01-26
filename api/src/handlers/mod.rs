use core::application::http::error::ApiError;

use axum::Extension;
use beep_auth::domain::models::Identity;
use beep_server::http::response::Response;
use serde::Serialize;

pub mod notification;

#[derive(Serialize, PartialEq)]
pub struct HelloResponse {
    pub message: String,
    pub user_id: String,
    pub username: Option<String>,
}

pub async fn hello(
    Extension(identity): Extension<Identity>,
) -> Result<Response<HelloResponse>, ApiError> {
    Ok(Response::OK(HelloResponse {
        message: "Hello, World!".to_string(),
        user_id: identity.id().to_string(),
        username: Some(identity.username().to_string()),
    }))
}

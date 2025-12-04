use axum::Extension;
use beep_auth::domain::models::Identity;
use beep_server::{ApiError, http::response::Response};
use serde::Serialize;

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

pub async fn get_notifications() -> Result<Response<String>, ApiError> {
    Ok(Response::OK("List of notifications".to_string()))
}
use core::{application::services::get_notifications_for_user, domain::entities::notification::Notification};

use axum::{Extension, extract::{Path, State}};
use beep_auth::domain::models::Identity;
use beep_server::{ApiError, http::response::Response};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize, PartialEq)]
pub struct HelloResponse {
    pub message: String,
    pub user_id: String,
    pub username: Option<String>,
}

#[derive(Serialize, PartialEq)]
pub struct NotificationResponse {
    pub notifications: Vec<Notification>,
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

pub async fn get_notifications(
    Path(user_id): Path<String>,
    Extension(identity): Extension<Identity>,
    State(state): State<AppState>,
) -> Result<Response<NotificationResponse>, ApiError> {
    let resp = get_notifications_for_user(&state.service, identity, &user_id).await?;
    Ok(Response::OK(NotificationResponse { notifications: resp }))
}
use core::{
    application::{http::ValidateJson, services::{get_notifications_for_user, mark_notification_as_read}},
    domain::entities::notification::Notification,
};

use axum::{
    Extension,
    extract::{Path, State},
};
use beep_auth::domain::models::Identity;
use beep_server::{ApiError, http::response::Response};
use serde::{Deserialize, Serialize};
use validator::Validate;

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

#[derive(Validate, Deserialize)]
pub struct ReadNotificationsInput {
    #[validate(length(min = 1, message = "ids cannot be empty"))]
    pub notification_ids: Vec<String>,
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
    Ok(Response::OK(NotificationResponse {
        notifications: resp,
    }))
}

pub async fn read_notification(
    Path((user_id, notification_id)): Path<(String, String)>,
    Extension(identity): Extension<Identity>,
    State(state): State<AppState>,
) -> Result<Response<()>, ApiError> {
    mark_notification_as_read(&state.service, identity, user_id, notification_id).await?;
    Ok(Response::Accepted(()))
}

#[axum::debug_handler]
pub async fn read_notifications(
    Path(user_id): Path<String>,
    Extension(identity): Extension<Identity>,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<ReadNotificationsInput>,
) -> Result<Response<()>, ApiError> {
    // For each id in body, mark as read
    for notification_id in  payload.notification_ids {
        mark_notification_as_read(&state.service, identity.clone(), user_id.clone(), notification_id).await?;
    }
    Ok(Response::Accepted(()))
}
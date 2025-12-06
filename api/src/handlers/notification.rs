use core::{
    application::http::{ValidateJson, error::ApiError},
    domain::{
        entities::{notification::Notification, preference::NotificationPreference},
        ports::notification::NotificationService,
    },
};

use axum::{
    Extension,
    extract::{Path, State},
};
use beep_auth::domain::models::Identity;
use beep_server::http::response::Response;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::state::AppState;

#[derive(Serialize, PartialEq)]
pub struct NotificationResponse {
    pub notifications: Vec<Notification>,
}

#[derive(Validate, Deserialize)]
pub struct ReadNotificationsInput {
    #[validate(length(min = 1, message = "ids cannot be empty"))]
    pub notification_ids: Vec<String>,
}

#[derive(Serialize, PartialEq)]
pub struct NotificationPreferencesResponse {
    pub notifications_preferences: Vec<NotificationPreference>,
}

pub async fn get_notifications(
    Path(user_id): Path<String>,
    Extension(identity): Extension<Identity>,
    State(state): State<AppState>,
) -> Result<Response<NotificationResponse>, ApiError> {
    let resp = state
        .service
        .get_notifications_for_user(identity, &user_id)
        .await?;
    Ok(Response::OK(NotificationResponse {
        notifications: resp,
    }))
}

pub async fn read_notification(
    Path((user_id, notification_id)): Path<(String, String)>,
    Extension(identity): Extension<Identity>,
    State(state): State<AppState>,
) -> Result<Response<()>, ApiError> {
    state
        .service
        .mark_notification_as_read(identity, user_id, notification_id)
        .await?;
    Ok(Response::Accepted(()))
}

pub async fn read_notifications(
    Path(user_id): Path<String>,
    Extension(identity): Extension<Identity>,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<ReadNotificationsInput>,
) -> Result<Response<()>, ApiError> {
    // For each id in body, mark as read
    for notification_id in payload.notification_ids {
        state
            .service
            .mark_notification_as_read(identity.clone(), user_id.clone(), notification_id)
            .await?;
    }
    Ok(Response::Accepted(()))
}

pub async fn get_notification_preferences(
    Path(user_id): Path<String>,
    Extension(identity): Extension<Identity>,
    State(state): State<AppState>,
) -> Result<Response<NotificationPreferencesResponse>, ApiError> {
    let resp = state.service.get_preferences(identity, user_id).await?;
    Ok(Response::OK(NotificationPreferencesResponse {
        notifications_preferences: resp,
    }))
}

#[axum::debug_handler]
pub async fn update_notification_preferences(
    Path(user_id): Path<String>,
    Extension(identity): Extension<Identity>,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<NotificationPreference>,
) -> Result<Response<()>, ApiError> {
    state
        .service
        .update_preferences(identity, user_id, payload)
        .await?;

    Ok(Response::Accepted(()))
}

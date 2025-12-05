use axum::{
    Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{Next, from_fn_with_state},
    response::Response,
    routing::{get, patch, post},
};
use beep_server::{ApiError, http::auth_middleware};
use tracing::info_span;

use crate::{
    handlers::{
        get_notification_preferences, get_notifications, hello, read_notification,
        read_notifications,
    },
    state::AppState,
};

async fn service_auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    auth_middleware(State(state.service), req, next).await
}

pub fn router(state: AppState) -> Result<Router, ApiError> {
    let trace_layer =
        tower_http::trace::TraceLayer::new_for_http().make_span_with(|request: &Request| {
            let uri: String = request.uri().to_string();
            info_span!("http_request", method = ?request.method(), uri)
        });

    let router = Router::new()
        .route("/", get(hello))
        .route("/users/{user_id}/notifications", get(get_notifications))
        .route(
            "/users/{user_id}/notifications/{notification_id}/read",
            patch(read_notification),
        )
        .route(
            "/users/{user_id}/notifications/read",
            post(read_notifications),
        )
        .route(
            "/users/{userId}/notifications/preferences",
            get(get_notification_preferences),
        )
        // .route("/users/{userId}/notifications/preferences", patch(update_notification_preferences))
        .layer(trace_layer)
        .layer(from_fn_with_state(state.clone(), service_auth_middleware))
        .with_state(state);

    Ok(router)
}

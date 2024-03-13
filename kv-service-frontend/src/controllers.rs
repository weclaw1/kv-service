use std::sync::Arc;

use axum::{
    routing::{delete, get, put},
    Router,
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::services::key_value_service::KeyValueService;

pub mod key_value_controller;

#[derive(Clone)]
pub struct AppState {
    pub key_value_service: Arc<dyn KeyValueService>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/:key", get(key_value_controller::get_value))
        .route("/api/:key", put(key_value_controller::put_value))
        .route("/api/:key", delete(key_value_controller::delete_value))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(state)
}

mod info;
mod activity;

use std::sync::Arc;

use axum::{routing::get, Router};
use tokio::sync::RwLock;

use crate::state::PhixivState;

use self::info::artwork_info_handler;
use self::activity::activity_handler;

pub fn api_router(state: Arc<RwLock<PhixivState>>) -> Router<Arc<RwLock<PhixivState>>> {
    Router::new()
        .route("/info", get(artwork_info_handler))
        .route("/v1/statuses/:id", get(activity_handler))
        .with_state(state.clone())
}

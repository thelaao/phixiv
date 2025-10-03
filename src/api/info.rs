use std::sync::Arc;

use axum::{
    extract::{Host, Query, State},
    Json,
};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::{helper::PhixivError, pixiv::ArtworkListing, state::PhixivState};

#[derive(Deserialize)]
pub struct ArtworkInfoPath {
    pub language: Option<String>,
    pub id: String,
    pub index: Option<usize>,
}

pub(super) async fn artwork_info_handler(
    State(state): State<Arc<RwLock<PhixivState>>>,
    Query(path): Query<ArtworkInfoPath>,
    Host(host): Host,
) -> Result<Json<ArtworkListing>, PhixivError> {
    let state = state.read().await;

    Ok(Json(
        ArtworkListing::get_listing(
            path.language.unwrap_or_else(|| "jp".to_string()),
            path.id,
            path.index.unwrap_or_else(|| 0),
            &host,
            &state.client,
        )
        .await?,
    ))
}

use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};

use crate::helper::PhixivError;

#[derive(Deserialize)]
pub struct ArtworkInfoPath {
    pub language: Option<String>,
    pub id: String,
}

#[derive(Serialize, Clone)]
pub struct APIResponse {
    pub message: String,
}

pub(super) async fn artwork_info_handler(
    Query(path): Query<ArtworkInfoPath>,
) -> Result<Json<APIResponse>, PhixivError> {
    let message = format!(
        "The phixiv API is no longer available, you can call the Pixiv API directly, it has the same information. Example: https://www.pixiv.net/ajax/illust/{}?lang={}",
        path.id,
        path.language.unwrap_or_else(|| "jp".to_string())
    );

    Ok(Json(APIResponse { message }))
}

use std::{sync::Arc, time::Duration, env};

use axum::{
    body::StreamBody,
    extract::{Path, State},
    headers::CacheControl,
    response::IntoResponse,
    routing::get,
    Router, TypedHeader,
};
use tokio::sync::RwLock;

use crate::{
    helper::{self, PhixivError},
    state::PhixivState,
};

async fn proxy_handler(
    State(state): State<Arc<RwLock<PhixivState>>>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, PhixivError> {
    let state = state.read().await;

    let base = env::var("PXIMG_BASE").unwrap_or_else(|_| String::from("https://i.pximg.net/"));
    let url = format!("{base}{path}");

    let mut headers = helper::headers();
    headers.append("Referer", "https://www.pixiv.net/".parse()?);

    let response = state.client.get(&url).headers(headers).send().await?;

    Ok((
        response.status(),
        TypedHeader(
            CacheControl::new()
                .with_max_age(Duration::from_secs(60 * 60 * 24))
                .with_public(),
        ),
        StreamBody::new(response.bytes_stream()),
    ))
}

pub fn proxy_router(state: Arc<RwLock<PhixivState>>) -> Router<Arc<RwLock<PhixivState>>> {
    Router::new()
        .route("/*path", get(proxy_handler))
        .with_state(state)
}

use std::{env, sync::Arc};

use axum::{
    extract::{Host, OriginalUri, Path, Query, State},
    headers::{CacheControl, UserAgent},
    middleware,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router, TypedHeader,
};
use http::Uri;
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::{
    helper::PhixivError,
    pixiv::{ArtworkListing, ArtworkPath, RawArtworkPath},
    state::{authorized_middleware, PhixivState},
    activity::activity_handler,
};

async fn artwork_response(
    raw_path: RawArtworkPath,
    state: Arc<RwLock<PhixivState>>,
    host: String,
) -> anyhow::Result<Response> {
    let path: ArtworkPath = raw_path.try_into()?;

    let state = state.read().await;

    let listing = ArtworkListing::get_listing(
        path.language,
        path.id,
        &state.auth.access_token,
        &host,
        &state.client,
    )
    .await?;

    let artwork = listing.to_template(path.image_index, host).unwrap();

    Ok((
        TypedHeader(CacheControl::new().with_no_cache()),
        Html(artwork),
    )
        .into_response())
}

async fn artwork_handler(
    Path(path): Path<RawArtworkPath>,
    State(state): State<Arc<RwLock<PhixivState>>>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    Host(host): Host,
) -> Result<Response, PhixivError> {
    if let Some(resp) = filter_bots(user_agent, &path) {
        return Ok(resp);
    }

    Ok(artwork_response(path, state, host).await?)
}

#[derive(Deserialize)]
struct MemberIllustParams {
    pub illust_id: String,
}

impl From<MemberIllustParams> for RawArtworkPath {
    fn from(params: MemberIllustParams) -> Self {
        Self {
            language: None,
            id: params.illust_id,
            image_index: None,
        }
    }
}

async fn member_illust_handler(
    Query(params): Query<MemberIllustParams>,
    State(state): State<Arc<RwLock<PhixivState>>>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    Host(host): Host,
) -> Result<Response, PhixivError> {
    let raw_path: RawArtworkPath = params.into();

    if let Some(resp) = filter_bots(user_agent, &raw_path) {
        return Ok(resp);
    }

    Ok(artwork_response(raw_path, state, host).await?)
}

fn filter_bots(user_agent: UserAgent, raw_path: &RawArtworkPath) -> Option<Response> {
    if env::var("BOT_FILTERING")
        .unwrap_or_else(|_| String::from("false"))
        .parse::<bool>()
        .ok()?
    {
        let bots = isbot::Bots::default();

        if !bots.is_bot(user_agent.as_str()) {
            let redirect_uri = format!(
                "https://www.pixiv.net{}/artworks/{}",
                raw_path
                    .language
                    .as_ref()
                    .map(|l| format!("/{l}"))
                    .unwrap_or_else(|| String::from("")),
                raw_path.id
            );
            return Some(Redirect::temporary(&redirect_uri).into_response());
        }
    }

    None
}

fn redirect_uri(uri: Uri) -> String {
    let Some(path_and_query) = uri.path_and_query() else {
        return String::from("https://www.pixiv.net/");
    };

    Uri::builder()
        .scheme("https")
        .authority("www.pixiv.net")
        .path_and_query(path_and_query.as_str())
        .build()
        .unwrap()
        .to_string()
}

async fn redirect_fallback(OriginalUri(uri): OriginalUri) -> Redirect {
    Redirect::temporary(&redirect_uri(uri))
}

pub fn router(
    state: Arc<RwLock<PhixivState>>,
) -> Router<Arc<RwLock<PhixivState>>, axum::body::Body> {
    Router::new()
        .route("/:language/artworks/:id", get(artwork_handler))
        .route("/:language/artworks/:id/:image_index", get(artwork_handler))
        .route("/artworks/:id", get(artwork_handler))
        .route("/artworks/:id/:image_index", get(artwork_handler))
        .route("/member_illust.php", get(member_illust_handler))
        .route("/api/v1/statuses/:id", get(activity_handler))
        .fallback(redirect_fallback)
        .layer(middleware::from_fn_with_state(state, authorized_middleware))
}

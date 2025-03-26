use std::sync::Arc;

use axum::{
    extract::{Host, Path, Query, State},
    Json,
};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    helper::PhixivError,
    pixiv::ArtworkListing,
    state::PhixivState,
};

#[derive(Deserialize)]
pub struct ActivityParams {
    pub id: String,
}

#[derive(Deserialize)]
pub struct ActivityQuery {
    pub language: Option<String>,
    pub image_index: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActivityResponse {
    id: String, // variable
    url: String, // variable
    uri: String, // variable same as url
    created_at: String, // variable
    edited_at: Option<serde_json::Value>,
    reblog: Option<serde_json::Value>,
    language: String,
    content: String, // variable
    spoiler_text: String,
    visibility: String,
    application: Application,
    media_attachments: Vec<MediaAttachment>,
    account: Account,
    mentions: Vec<serde_json::Value>,
    tags: Vec<serde_json::Value>,
    emojis: Vec<serde_json::Value>,
    card: Option<serde_json::Value>,
    poll: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Application {
    name: String,
    website: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MediaAttachment {
    id: String, // variable
    #[serde(rename = "type")]
    media_type: String,
    url: String, // variable
    preview_url: String, // variable same as url
    remote_url: Option<serde_json::Value>,
    preview_remote_url: Option<serde_json::Value>,
    text_url: Option<serde_json::Value>,
    description: String,
    meta: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    id: String, // variable
    display_name: String, // variable
    username: String, // variable
    acct: String, // variable same as display_name
    url: String, // variable
    uri: String, // variable same as url
    created_at: String, // variable
    locked: bool,
    bot: bool,
    discoverable: bool,
    indexable: bool,
    group: bool,
    avatar: Option<String>, // variable
    avatar_static: Option<String>, // variable same as avatar
    header: Option<serde_json::Value>, // variable
    header_static: Option<serde_json::Value>, // variable same as header
    followers_count: i64,
    following_count: i64,
    statuses_count: i64,
    hide_collections: bool,
    noindex: bool,
    emojis: Vec<serde_json::Value>,
    roles: Vec<serde_json::Value>,
    fields: Vec<serde_json::Value>,
}

impl ActivityResponse {
    fn new(
        id: String,
        url: String,
        created_at: String,
        content: String,
        media_attachment_url: String,
        account_id: String,
        account_display_name: String,
        account_avatar: Option<String>,
    ) -> Self {
        Self {
            id: id.clone(),
            url: url.clone(),
            uri: url.clone(),
            created_at: created_at.clone(),
            edited_at: None,
            reblog: None,
            language: "en".to_string(),
            content,
            spoiler_text: "".to_string(),
            visibility: "public".to_string(),
            application: Application {
                name: "Twitter Web App".to_string(),
                website: None,
            },
            media_attachments: vec![MediaAttachment {
                id: id,
                media_type: "image".to_string(),
                url: media_attachment_url.clone(),
                preview_url: media_attachment_url,
                remote_url: None,
                preview_remote_url: None,
                text_url: None,
                description: "".to_string(),
                meta: serde_json::json!({}),
            }],
            account: Account {
                id: account_id,
                display_name: account_display_name.clone(),
                username: account_display_name.clone(),
                acct: account_display_name,
                url: url.clone(),
                uri: url,
                created_at,
                locked: false,
                bot: false,
                discoverable: true,
                indexable: false,
                group: false,
                avatar: account_avatar.clone(),
                avatar_static: account_avatar,
                header: None,
                header_static: None,
                followers_count: 0,
                following_count: 0,
                statuses_count: 0,
                hide_collections: false,
                noindex: false,
                emojis: vec![],
                roles: vec![],
                fields: vec![],
            },
            mentions: vec![],
            tags: vec![],
            emojis: vec![],
            card: None,
            poll: None,
        }
    }
}

pub async fn activity_handler(
    Path(path): Path<ActivityParams>,
    Query(query): Query<ActivityQuery>,
    State(state): State<Arc<RwLock<PhixivState>>>,
    Host(host): Host,
) -> Result<Json<ActivityResponse>, PhixivError> {
    let state = state.read().await;
    let listing = ArtworkListing::get_listing(
        query.language,
        path.id.clone(),
        &host,
        &state.client,
    )
    .await?;

    let created_at = DateTime::parse_from_rfc3339(&listing.create_date).unwrap().to_utc().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let index = query.image_index
        .unwrap_or(1)
        .min(listing.image_proxy_urls.len())
        .saturating_sub(1);
    let image_url = listing.image_proxy_urls[index].clone();

    Ok(Json(ActivityResponse::new(
        path.id,
        listing.url,
        created_at,
        listing.description,
        image_url,
        listing.author_id,
        listing.author_name,
        listing.profile_image_url,
    )))
}

use std::sync::Arc;

use axum::{
    extract::{Host, Path, State},
    Json,
};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use itertools::Itertools;

use crate::{
    helper::{PhixivError, ActivityId},
    pixiv::ArtworkListing,
    state::PhixivState,
};

#[derive(Deserialize)]
pub struct ActivityParams {
    pub id: String,
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
        created_at: String,
        media_attachment_url: String,
        media_attachment_preview_url: String,
        listing: ArtworkListing,
    ) -> Self {
        let tag_string = Itertools::intersperse_with(listing.tags.into_iter(), || String::from(", "))
            .collect::<String>();

        let description = Itertools::intersperse_with(
            [
                format!("<strong><a href=\"{}\">{}</a></strong>", listing.url, listing.title),
                String::from(if listing.ai_generated {
                    "<strong>AI Generated</strong><br />"
                } else {
                    ""
                }),
                listing.description,
                tag_string.clone(),
            ]
            .into_iter()
            .filter(|s| !s.is_empty()),
            || String::from("<br />"),
        )
        .collect::<String>();

        let media_type = {
            if media_attachment_url.contains("ugoira") {
                "video"
            } else {
                "image"
            }
        }.to_string();

        Self {
            id: id.clone(),
            url: listing.url.clone(),
            uri: listing.url.clone(),
            created_at: created_at.clone(),
            edited_at: None,
            reblog: None,
            language: "en".to_string(),
            content: description,
            spoiler_text: "".to_string(),
            visibility: "public".to_string(),
            application: Application {
                name: "Pixiv".to_string(),
                website: None,
            },
            media_attachments: vec![MediaAttachment {
                id: id,
                media_type: media_type,
                url: media_attachment_url,
                preview_url: media_attachment_preview_url,
                remote_url: None,
                preview_remote_url: None,
                text_url: None,
                description: "".to_string(),
                meta: serde_json::json!({}),
            }],
            account: Account {
                id: listing.author_id,
                display_name: listing.author_name,
                username: "".to_string(),
                acct: "".to_string(),
                url: listing.url.clone(),
                uri: listing.url,
                created_at,
                locked: false,
                bot: false,
                discoverable: true,
                indexable: false,
                group: false,
                avatar: listing.profile_image_url.clone(),
                avatar_static: listing.profile_image_url,
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
    State(state): State<Arc<RwLock<PhixivState>>>,
    Host(host): Host,
) -> Result<Json<ActivityResponse>, PhixivError> {
    let activity_id: u64 = path.id.parse()?;
    let activity_id = ActivityId::from(activity_id);

    let state = state.read().await;
    let listing = ArtworkListing::get_listing(
        activity_id.language,
        activity_id.id.to_string(),
        &host,
        &state.client,
    )
    .await?;

    let created_at = DateTime::parse_from_rfc3339(&listing.create_date).unwrap().to_utc().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let index = (activity_id.index as usize).min(listing.image_proxy_urls.len().saturating_sub(1));
    let image_url = listing.image_proxy_urls[index].clone();
    let preview_url = if image_url.contains("ugoira") {
        listing.image_proxy_urls[1].clone()
    } else {
        image_url.clone()
    };

    Ok(Json(ActivityResponse::new(
        activity_id.id.to_string(),
        created_at,
        image_url,
        preview_url,
        listing,
    )))
}

use std::env;

use askama::Template;
use http::HeaderMap;
use itertools::Itertools;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use self::model::AjaxResponse;
use crate::helper::ActivityId;

mod model;

#[derive(Deserialize)]
pub struct RawArtworkPath {
    pub language: Option<String>,
    pub id: String,
    pub image_index: Option<String>,
}

pub struct ArtworkPath {
    pub language: Option<String>,
    pub id: String,
    pub image_index: Option<usize>,
}

impl TryFrom<RawArtworkPath> for ArtworkPath {
    type Error = anyhow::Error;

    fn try_from(value: RawArtworkPath) -> Result<Self, Self::Error> {
        let image_index = match value.image_index {
            Some(index) => Some(index.chars().take_while(|c| c.is_numeric()).collect::<String>().parse()?),
            None => None,
        };

        Ok(Self {
            language: value.language,
            id: value.id,
            image_index,
        })
    }
}

#[derive(Debug, Serialize, Template)]
#[template(path = "artwork.html")]
pub struct ArtworkTemplate {
    pub image_proxy_url: String,
    pub title: String,
    pub description: String,
    pub author_name: String,
    pub author_id: String,
    pub url: String,
    pub alt_text: String,
    pub host: String,
    pub activity_id: u64,
    pub site_name: String,
}

#[derive(Debug, Serialize, Template)]
#[template(path = "ugoira.html")]
pub struct UgoiraTemplate {
    pub image_proxy_url: String,
    pub title: String,
    pub description: String,
    pub author_name: String,
    pub author_id: String,
    pub url: String,
    pub alt_text: String,
    pub host: String,
    pub site_name: String,
}

#[derive(Serialize)]
/// Representing a listing of artworks, uniquely determined by language and illust_id
pub struct ArtworkListing {
    pub image_proxy_urls: Vec<String>,
    pub title: String,
    pub ai_generated: bool,
    pub description: String,
    pub tags: Vec<String>,
    pub url: String,
    pub author_name: String,
    pub author_id: String,
    pub is_ugoira: bool,
    pub create_date: String,
    pub illust_id: String,
    pub profile_image_url: Option<String>,
    pub language: String,
}

async fn ajax_request(
    illust_id: &String,
    language: &Option<String>,
    client: &Client,
) -> anyhow::Result<AjaxResponse> {
    let mut ajax_headers = HeaderMap::with_capacity(2);
    if let Ok(pixiv_cookie) = env::var("PIXIV_COOKIE") {
        ajax_headers.append("Cookie", format!("PHPSESSID={}", pixiv_cookie).parse()?);
    }
    ajax_headers.append("User-Agent", env::var("USER_AGENT").unwrap_or_else(|_| {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36".to_string()
    }).parse()?);

    Ok(client
        .get(format!(
            "https://www.pixiv.net/ajax/illust/{}?lang={}",
            &illust_id,
            &language.clone().unwrap_or_else(|| String::from("jp"))
        ))
        .headers(ajax_headers)
        .send()
        .await?
        .json()
        .await?)
}

impl ArtworkListing {
    pub async fn get_listing(
        language: Option<String>,
        illust_id: String,
        host: &str,
        client: &Client,
    ) -> anyhow::Result<Self> {
        let clean_illust_id = illust_id.chars().take_while(|c| c.is_numeric()).collect::<String>();
        let ajax_response = ajax_request(&clean_illust_id, &language, client).await?;

        let ai_generated = ajax_response.body.ai_type == 2;

        let raw_profile_image_url = ajax_response.body.user_illusts.iter()
            .filter_map(|(_, user_illust_option)| user_illust_option.as_ref())
            .filter_map(|user_illust| user_illust.profile_image_url.clone())
            .next();
        let profile_image_url = raw_profile_image_url.and_then(|raw_url| {
            url::Url::parse(&raw_url)
            .ok()
            .map(|parsed_url| format!("https://{}/i{}", host, parsed_url.path()))
        });

        let tags: Vec<_> = ajax_response.body
            .tags
            .tags
            .into_iter()
            .map(|tag| {
                format!(
                    "#{}",
                    if let Some(language) = &language {
                        if let Some(translation) = tag.translation {
                            translation.get(language).unwrap_or(&tag.tag).to_string()
                        } else {
                            tag.tag
                        }
                    } else {
                        tag.tag
                    }
                )
            })
            .collect();

        let is_ugoira = ajax_response.body.illust_type == 2;
        let ugoira_enabled = env::var("UGOIRA_ENABLED")
            .unwrap_or_else(|_| String::from("false")) == "true";
        let image_url = ajax_response.body.urls.regular.or(ajax_response.body.urls.original).unwrap();

        let image_proxy_urls = if is_ugoira && ugoira_enabled {
            vec![format!("https://{}/i/ugoira/{}.mp4", host, clean_illust_id)]
        } else {
            let path = url::Url::parse(&image_url)?.path().to_string();
            (0..ajax_response.body.page_count).map(|i| {
                let current_path = if i == 0 {
                    path.clone()
                } else {
                    path.replace("_p0_", &format!("_p{}_", i))
                };
                format!("https://{}/i{}", host, current_path)
            }).collect::<Vec<String>>()
        };

        let language = language.unwrap_or_else(|| "jp".to_string());

        Ok(Self {
            image_proxy_urls,
            title: ajax_response.body.title,
            ai_generated,
            description: ajax_response.body.description,
            tags,
            url: ajax_response.body.extra_data.meta.canonical,
            author_name: ajax_response.body.author_name,
            author_id: ajax_response.body.author_id,
            is_ugoira,
            create_date: ajax_response.body.create_date,
            illust_id,
            profile_image_url,
            language,
        })
    }

    pub fn to_template(self, image_index: Option<usize>, host: String) -> anyhow::Result<String> {
        let index = image_index
            .unwrap_or(1)
            .min(self.image_proxy_urls.len())
            .saturating_sub(1);

        let image_proxy_url = self.image_proxy_urls[index].clone();

        let tag_string = Itertools::intersperse_with(self.tags.into_iter(), || String::from(", "))
            .collect::<String>();

        let description = Itertools::intersperse_with(
            [
                String::from(if self.ai_generated {
                    "AI Generated\n"
                } else {
                    ""
                }),
                self.description,
                tag_string.clone(),
            ]
            .into_iter()
            .filter(|s| !s.is_empty()),
            || String::from("\n"),
        )
        .collect::<String>();

        let activity_id = u64::from(ActivityId {
            language: self.language,
            id: self.illust_id.parse()?,
            index: index as u16,
        });

        let site_name = env::var("PROVIDER_NAME").unwrap_or_else(|_| String::from("phixiv"));

        if self.is_ugoira {
            let template = UgoiraTemplate {
                image_proxy_url,
                title: self.title,
                description,
                author_name: self.author_name,
                author_id: self.author_id,
                url: self.url,
                alt_text: tag_string,
                host,
                site_name,
            };
            return Ok(template.render()?);
        }
        let template = ArtworkTemplate {
            image_proxy_url,
            title: self.title,
            description,
            author_name: self.author_name,
            author_id: self.author_id,
            url: self.url,
            alt_text: tag_string,
            host,
            activity_id,
            site_name,
        };
        Ok(template.render()?)
    }
}

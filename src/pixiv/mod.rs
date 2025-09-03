use std::env;

use askama::Template;
use cached::proc_macro::cached;
use cached::SizedCache;
use fancy_regex::{Captures, Regex};
use http::HeaderMap;
use itertools::Itertools;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use self::model::AjaxResponse;
use crate::helper::{provider_name, ActivityId};

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
            Some(index) => Some(
                index
                    .chars()
                    .take_while(|c| c.is_numeric())
                    .collect::<String>()
                    .parse()?,
            ),
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
    pub activity_id: u64,
    pub site_name: String,
}

#[derive(Serialize, Clone)]
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
    pub bookmark_count: u32,
    pub like_count: u32,
    pub comment_count: u32,
    pub view_count: u32,
    pub x_restrict: u32,
}

async fn ajax_request(
    illust_id: &String,
    language: &String,
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
            &illust_id, &language
        ))
        .headers(ajax_headers)
        .send()
        .await?
        .json()
        .await?)
}

#[cached(
    ty = "SizedCache<String, ArtworkListing>",
    create = "{ SizedCache::with_size(1024) }",
    convert = r#"{ format!("{}_{}", language, illust_id) }"#,
    result = true
)]
async fn cached_get_listing(
    language: String,
    illust_id: String,
    host: &str,
    client: &Client,
) -> anyhow::Result<ArtworkListing> {
    let clean_illust_id = illust_id
        .chars()
        .take_while(|c| c.is_numeric())
        .collect::<String>();
    let ajax_response = ajax_request(&clean_illust_id, &language, client).await?;

    let ai_generated = ajax_response.body.ai_type == 2;

    let raw_profile_image_url = ajax_response
        .body
        .user_illusts
        .iter()
        .filter_map(|(_, user_illust_option)| user_illust_option.as_ref())
        .filter_map(|user_illust| user_illust.profile_image_url.clone())
        .next();
    let profile_image_url = raw_profile_image_url.and_then(|raw_url| {
        url::Url::parse(&raw_url)
            .ok()
            .map(|parsed_url| format!("https://{}/i{}", host, parsed_url.path()))
    });

    let tags: Vec<_> = ajax_response
        .body
        .tags
        .tags
        .into_iter()
        .map(|tag| {
            format!(
                "#{}",
                if let Some(translation) = tag.translation {
                    translation.get(&language).unwrap_or(&tag.tag).to_string()
                } else {
                    tag.tag
                }
            )
        })
        .collect();

    let is_ugoira = ajax_response.body.illust_type == 2;
    let ugoira_enabled =
        env::var("UGOIRA_ENABLED").unwrap_or_else(|_| String::from("false")) == "true";
    let image_url = ajax_response
        .body
        .urls
        .regular
        .or(ajax_response.body.urls.original)
        .unwrap();
    let path = url::Url::parse(&image_url)?.path().to_string();
    let thumbnail_type = env::var("THUMBNAIL_TYPE").ok();

    let image_proxy_urls = if is_ugoira && ugoira_enabled {
        vec![
            format!("https://{}/i/ugoira/{}.mp4", host, clean_illust_id),
            format!("https://{}/i{}", host, path),
        ]
    } else {
        (0..ajax_response.body.page_count)
            .map(|i| {
                let mut current_path = if i == 0 {
                    path.clone()
                } else {
                    path.replace("_p0_", &format!("_p{}_", i))
                };
                if let Some(replacement) = &thumbnail_type {
                    current_path = current_path.replace("img-master", replacement);
                }
                format!("https://{}/i{}", host, current_path)
            })
            .collect::<Vec<String>>()
    };
    let description = fix_links(ajax_response.body.description);

    Ok(ArtworkListing {
        image_proxy_urls,
        title: ajax_response.body.title,
        ai_generated,
        description: description,
        tags,
        url: ajax_response.body.extra_data.meta.canonical,
        author_name: ajax_response.body.author_name,
        author_id: ajax_response.body.author_id,
        is_ugoira,
        create_date: ajax_response.body.create_date,
        illust_id: clean_illust_id,
        profile_image_url,
        language,
        bookmark_count: ajax_response.body.bookmark_count,
        like_count: ajax_response.body.like_count,
        comment_count: ajax_response.body.comment_count,
        view_count: ajax_response.body.view_count,
        x_restrict: ajax_response.body.x_restrict,
    })
}

fn fix_links(description: String) -> String {
    let re = Regex::new("href=\"/jump.php\\?(.*?)\"").unwrap();
    re.replace_all(&description, |caps: &Captures| {
        format!("href=\"{}\"", urlencoding::decode(&caps[1]).unwrap())
    })
    .into_owned()
}

impl ArtworkListing {
    pub async fn get_listing(
        language: String,
        illust_id: String,
        host: &str,
        client: &Client,
    ) -> anyhow::Result<Self> {
        cached_get_listing(language, illust_id, host, client).await
    }

    pub fn to_template(self, image_index: Option<usize>, host: String) -> anyhow::Result<String> {
        let index = if self.is_ugoira {
            0
        } else {
            image_index
                .unwrap_or(1)
                .min(self.image_proxy_urls.len())
                .saturating_sub(1)
        };

        let image_proxy_url = self.image_proxy_urls[index].clone();

        let tag_string = Itertools::intersperse_with(self.tags.into_iter(), || String::from(", "))
            .collect::<String>();

        let description_text = if host.starts_with("c.") {
            String::new()
        } else {
            Self::extract_html_inner_text(self.description)
        };
        let description = Itertools::intersperse_with(
            [
                format!(
                    "{}{}",
                    match self.ai_generated {
                        true => String::from("[AI Generated] "),
                        false => String::new(),
                    },
                    description_text
                ),
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

        let site_name = provider_name();

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
                activity_id,
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

    /// Extract visible strings (innerText) from html string.
    ///
    /// The html flavor is based on documentation from *pixiv Help Center*: [What is a caption?](https://www.pixiv.help/hc/en-us/articles/235646067-What-is-a-caption).
    /// There is NO any special processing for shorthand links: [I want to put a shorthand link to other illustrations and novels in the caption (like illust/○○○ and novel/○○○) when I post an illustration on pixiv](https://www.pixiv.help/hc/en-us/articles/235645647-I-want-to-put-a-shorthand-link-to-other-illustrations-and-novels-in-the-caption-like-illust-and-novel-when-I-post-an-illustration-on-pixiv)
    ///
    /// # Example
    ///
    /// ```rust
    /// let expected = vec![
    ///     "Caption: https://example.com/ a<NOT A TAG>",
    ///     "b_STRONG  I<x>I",
    ///     "S0",
    ///     "S1",
    ///     "https://example.com/",
    ///     "A More Com<>ple<x> One",
    /// ]
    /// .join("\n");
    ///
    /// let result = extract_html_inner_text(vec![
    ///     "    Caption:",
    ///     r#"<a href="/jump.php?https%3A%2F%2Fexample.com%2F" target="_blank">https://example.com/</a>"#,
    ///     "a<NOT A TAG><br />b",
    ///     r#"<span style="color:#fff;">_</span >"#,
    ///     "<strong>STRONG</strong  >",
    ///     "<i>  I<x>I  </i>",
    ///     "<br >",
    ///     "<s>S0<br>S1</s>",
    ///     "<empty></empty    >",
    ///     r#"<br  /><a>https://example.com/</a><br  />"#,
    ///     "<strong>A<i> More </i>Com<>ple<x> <s>One</s></strong>",
    ///     "    ",
    /// ]
    /// .join(""));
    ///
    /// assert_eq!(expected, result);
    /// ```
    fn extract_html_inner_text(html: String) -> String {
        let re = Regex::new(
            r"^(?<before>.*?)<(?<tag>[^\s>]+)(?:\s*[^>]+)?>(?<inner>.*?)</\k<tag>\s*>(?<after>[^$]*)$")
            .unwrap();

        let mut full_string: String = String::with_capacity(html.len());
        let mut string_segments = vec![html];

        while let Some(segment) = string_segments.pop() {
            full_string += match re.captures(&segment).unwrap() {
                Some(captures) => {
                    string_segments.push(String::from(captures.name("after").unwrap().as_str()));

                    let mut inner = String::from(captures.name("inner").unwrap().as_str());
                    if captures.name("tag").unwrap().as_str() == "a"
                    /* anchor */
                    {
                        // avoid unexpected concatenation
                        inner = format!(" {} ", inner)
                    }
                    string_segments.push(inner);

                    captures.name("before").unwrap().as_str()
                }
                None => segment.as_str(),
            }
        }

        Regex::new(r"<br\s*/?>")
            .unwrap()
            .split(&full_string)
            .map(|x| {
                String::from(
                    x.unwrap().trim(), /* for text from standalone anchors */
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}

use std::env;

use axum::response::{IntoResponse, Response};
use http::{HeaderMap, HeaderValue, StatusCode};

pub fn headers() -> HeaderMap<HeaderValue> {
    let mut headers = HeaderMap::with_capacity(5);

    headers.append("App-Os", "iOS".parse().unwrap());
    headers.append("App-Os-Version", "14.6".parse().unwrap());
    headers.append(
        "User-Agent",
        "PixivIOSApp/7.13.3 (iOS 14.6; iPhone13,2)".parse().unwrap(),
    );

    headers
}

pub fn provider_name() -> String {
    env::var("PROVIDER_NAME").unwrap_or_else(|_| String::from("phixiv"))
}

pub fn provider_url() -> String {
    env::var("PROVIDER_URL")
        .unwrap_or_else(|_| String::from("https://github.com/HazelTheWitch/phixiv"))
}

pub struct PhixivError(anyhow::Error);

impl IntoResponse for PhixivError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{:#}", self.0)).into_response()
    }
}

impl<E> From<E> for PhixivError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

#[derive(Debug)]
pub struct ActivityId {
    pub language: String,
    pub id: u32,
    pub index: u16,
    pub offset_end: u16,
}

impl From<u64> for ActivityId {
    fn from(value: u64) -> Self {
        let offset_end = (value >> 56) & 0xFF;
        let lang_id = (value >> 48) & 0xFF;
        let id = (value >> 16) & 0xFFFFFFFF;
        let index = value & 0xFFFF;

        let language = match lang_id {
            0 => "jp".to_string(),
            1 => "en".to_string(),
            2 => "zh".to_string(),
            3 => "zh_tw".to_string(),
            4 => "ko".to_string(),
            _ => "jp".to_string(),
        };

        Self {
            language,
            id: id as u32,
            index: index as u16,
            offset_end: offset_end as u16,
        }
    }
}

impl From<ActivityId> for u64 {
    fn from(value: ActivityId) -> Self {
        let lang_id = match value.language.as_str() {
            "jp" => 0,
            "en" => 1,
            "zh" => 2,
            "zh_tw" => 3,
            "ko" => 4,
            _ => 0,
        };

        (value.offset_end.min(0xFF) as u64) << 56
            | (lang_id as u64) << 48
            | (value.id as u64) << 16
            | (value.index as u64)
    }
}

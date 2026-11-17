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
    pub tl: bool,
}

impl From<u64> for ActivityId {
    fn from(value: u64) -> Self {
        let offset_end = (value >> 56) & 0xFF;
        let tl = (value >> 55) & 1 == 1;
        let lang_id = (value >> 48) & 0x7F;
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
            tl,
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
            | (value.tl as u64) << 55
            | (lang_id as u64) << 48
            | (value.id as u64) << 16
            | (value.index as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::ActivityId;

    #[test]
    fn legacy_activity_id_is_unchanged() {
        // Encoding produced before the tl flag existed: en (1), id 124748386, index 2.
        let legacy_id = (1u64) << 48 | (124748386u64) << 16 | 2;

        let activity_id = ActivityId::from(legacy_id);

        assert_eq!(activity_id.language, "en");
        assert_eq!(activity_id.id, 124748386);
        assert_eq!(activity_id.index, 2);
        assert!(!activity_id.tl);
        assert_eq!(u64::from(activity_id), legacy_id);
    }

    #[test]
    fn activity_id_round_trips_with_tl() {
        for (language, tl) in [("jp", true), ("en", true), ("zh_tw", false)] {
            let activity_id = ActivityId {
                language: language.to_string(),
                id: 124748386,
                index: 1,
                offset_end: 2,
                tl,
            };

            let decoded = ActivityId::from(u64::from(activity_id));

            assert_eq!(decoded.language, language);
            assert_eq!(decoded.id, 124748386);
            assert_eq!(decoded.index, 1);
            assert_eq!(decoded.offset_end, 2);
            assert_eq!(decoded.tl, tl);
        }
    }
}

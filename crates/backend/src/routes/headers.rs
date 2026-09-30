use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::Response;

use crate::error::AppError;

pub const CACHE_IMMUTABLE: &str = "private, max-age=31536000, immutable";
pub const CACHE_REVALIDATE: &str = "private, max-age=0, must-revalidate";
pub const CACHE_DAY: &str = "private, max-age=86400";
pub const CACHE_MINUTE: &str = "private, max-age=60";

pub fn attachment(filename: &str) -> Result<HeaderValue, AppError> {
    let ascii: String = filename
        .chars()
        .map(|c| {
            if (c.is_ascii_graphic() && c != '"' && c != '\\') || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let encoded: String = filename
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"!#$&+-.^_`|~".contains(&b) {
                char::from(b).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect();
    header_value(&format!(
        "attachment; filename=\"{ascii}\"; filename*=UTF-8''{encoded}"
    ))
}

pub fn etag(tag: &str) -> Result<HeaderValue, AppError> {
    header_value(&format!("\"{tag}\""))
}

pub fn etag_matches(headers: &HeaderMap, etag: &HeaderValue) -> bool {
    headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value
                .split(',')
                .any(|candidate| candidate.trim().as_bytes() == etag.as_bytes())
        })
}

pub fn attach_validators(resp: &mut Response, etag: HeaderValue) {
    resp.headers_mut().insert(header::ETAG, etag);
    resp.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(CACHE_REVALIDATE),
    );
}

pub fn accepts_zstd(headers: &HeaderMap) -> bool {
    headers
        .get_all(header::ACCEPT_ENCODING)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .any(|item| {
            let mut params = item.split(';');
            let coding = params.next().unwrap_or_default().trim();
            coding.eq_ignore_ascii_case("zstd")
                && params
                    .find_map(|param| {
                        let (name, value) = param.split_once('=')?;
                        name.trim().eq_ignore_ascii_case("q").then_some(value)
                    })
                    .is_none_or(|q| q.trim().parse::<f32>().is_ok_and(|q| q > 0.0))
        })
}

pub fn header_value(value: &str) -> Result<HeaderValue, AppError> {
    HeaderValue::from_str(value).map_err(|e| {
        tracing::error!(error = %e, "invalid response header value");
        AppError::Internal
    })
}

#[cfg(test)]
mod tests;

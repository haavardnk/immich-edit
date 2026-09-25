pub mod admin;
pub mod albums;
pub mod assets;
pub mod auth;
pub mod copies;
pub mod dcp;
pub mod debug;
pub mod edited_thumb;
pub mod edits;
pub mod export;
pub mod faces;
pub mod folders;
pub mod health;
pub mod jobs;
pub mod lens_profile;
pub mod luts;
#[cfg(feature = "ml")]
pub mod masks;
#[cfg(feature = "ml")]
pub mod models;
pub mod people;
pub mod presets;
pub mod preview;
pub mod rasters;
pub mod search;
pub mod setup;
pub mod source;
pub mod tags;
pub mod watermarks;

use axum::body::Body;
use axum::http::{HeaderValue, header};
use axum::response::Response;

pub(crate) fn immutable_bytes(bytes: Vec<u8>, content_type: &'static str) -> Response {
    let mut resp = Response::new(Body::from(bytes));
    let headers = resp.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=31536000, immutable"),
    );
    resp
}

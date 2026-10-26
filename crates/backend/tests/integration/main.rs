mod common;

mod admin;
mod albums;
mod assets;
mod auth_logout;
mod auth_oauth;
mod auth_setup;
mod client_log;
mod copies;
mod cross_user_isolation;
mod csrf;
mod debug;
#[cfg(feature = "ml")]
mod denoise;
#[cfg(feature = "ml")]
mod denoise_export;
#[cfg(feature = "ml")]
mod denoise_report;
#[cfg(feature = "ml")]
mod denoise_revision;
mod edits;
mod export;
mod export_immich;
mod export_presets;
mod export_replace;
mod faces;
mod folders;
mod frame_cache_isolation;
mod frame_single_flight;
mod frame_warm;
mod health;
mod immich_capabilities;
mod input_validation;
mod jobs;
mod jobs_lifecycle;
mod logging;
mod login_rate_limit;
mod middleware;
#[cfg(feature = "ml")]
mod models;
mod people;
mod presets;
mod preview;
mod rasters;
mod request_timeout;
mod search;
mod sessions;
mod setup_oauth;
mod source;
mod tags;
mod trash;
mod watermarks;

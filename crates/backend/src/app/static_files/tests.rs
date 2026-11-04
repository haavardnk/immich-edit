use super::{cache_policy, names_a_file};
use crate::routes::headers::{CACHE_IMMUTABLE, CACHE_REVALIDATE};

#[test]
fn paths_with_an_extension_name_a_file() {
    for path in [
        "/service-worker.js",
        "/missing.png",
        "/_app/immutable/chunks/app.B1x.js",
        "/fonts/GoogleSans[GRAD,opsz,wght].ttf",
    ] {
        assert!(names_a_file(path), "{path}");
    }
}

#[test]
fn client_routes_do_not_name_a_file() {
    for path in [
        "/",
        "/photos",
        "/assets/123",
        "/assets/0b7c6f1e-3c5a-4f1e-9a43-1d2b3c4d5e6f_2",
        "/.well-known",
        "/settings/",
        "/folders/trailing.",
        "/_app.d/route",
    ] {
        assert!(!names_a_file(path), "{path}");
    }
}

#[test]
fn only_hashed_bundle_output_is_immutable() {
    assert_eq!(cache_policy("/_app/immutable/chunks/a.js"), CACHE_IMMUTABLE);
    assert_eq!(cache_policy("/_app/version.json"), CACHE_REVALIDATE);
    assert_eq!(cache_policy("/_app/immutable"), CACHE_REVALIDATE);
    assert_eq!(cache_policy("/fonts/a.ttf"), CACHE_REVALIDATE);
    assert_eq!(cache_policy("/service-worker.js"), CACHE_REVALIDATE);
}

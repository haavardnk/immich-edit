use crate::names::display_name;

const MAX_URL_BYTES: usize = 2048;

pub fn name(raw: &str) -> Result<String, String> {
    display_name(raw, "saved view name")
}

pub fn url(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("saved view url required".into());
    }
    if url.len() > MAX_URL_BYTES {
        return Err(format!("saved view url longer than {MAX_URL_BYTES} bytes"));
    }
    if !url.bytes().all(|b| (b'!'..=b'~').contains(&b)) {
        return Err("saved view url must be printable ASCII without spaces".into());
    }
    if !url.starts_with('/') {
        return Err("saved view url must start with /".into());
    }
    if url.starts_with("//") || url.starts_with("/\\") {
        return Err("saved view url must be a path on this site".into());
    }
    if url.contains('\\') {
        return Err("saved view url must not contain a backslash".into());
    }
    if url.contains('#') {
        return Err("saved view url must not contain a fragment".into());
    }
    if path_of(url).split('/').any(is_dot_segment) {
        return Err("saved view url must not contain . or .. segments".into());
    }
    Ok(())
}

fn path_of(url: &str) -> &str {
    url.split_once('?').map_or(url, |(path, _)| path)
}

fn is_dot_segment(segment: &str) -> bool {
    matches!(
        segment.to_ascii_lowercase().replace("%2e", ".").as_str(),
        "." | ".."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url_error(candidate: &str) -> String {
        url(candidate).unwrap_err()
    }

    #[test]
    fn a_name_error_names_a_saved_view() {
        assert_eq!(name("   ").unwrap_err(), "saved view name required");
    }

    #[test]
    fn a_local_path_with_a_query_is_accepted() {
        for candidate in [
            "/",
            "/photos",
            "/photos?rating=3&sort=date",
            "/albums/0b1c2d3e-0000-4000-8000-000000000000?q=%20x",
            "/search?tags=a,b&people=c~d",
        ] {
            assert!(url(candidate).is_ok(), "{candidate}");
        }
    }

    #[test]
    fn a_url_is_bounded_in_bytes() {
        assert_eq!(url_error(""), "saved view url required");
        let at_limit = format!("/{}", "a".repeat(2047));
        assert_eq!(at_limit.len(), 2048);
        assert!(url(&at_limit).is_ok());
        let over = format!("/{}", "a".repeat(2048));
        assert!(url_error(&over).contains("longer than 2048"));
    }

    #[test]
    fn a_url_is_printable_ascii() {
        for candidate in ["/a b", "/a\tb", "/a\nb", "/a\u{7f}", "/photos/é", "/\u{0}"] {
            assert!(
                url_error(candidate).contains("printable ASCII"),
                "{candidate:?}"
            );
        }
    }

    #[test]
    fn a_url_starts_with_a_slash() {
        for candidate in ["photos", "https://evil.example/", "?rating=3"] {
            assert!(url_error(candidate).contains("start with /"), "{candidate}");
        }
    }

    #[test]
    fn a_url_cannot_leave_the_site() {
        for candidate in ["//x", "//evil.example/photos", "/\\x", "/\\evil.example"] {
            assert!(
                url_error(candidate).contains("path on this site"),
                "{candidate}"
            );
        }
    }

    #[test]
    fn a_url_has_no_backslash() {
        assert!(url_error("/a\\b").contains("backslash"));
        assert!(url_error("/a?q=\\").contains("backslash"));
    }

    #[test]
    fn a_url_has_no_dot_segments() {
        for candidate in [
            "/.",
            "/..",
            "/./photos",
            "/photos/../albums",
            "/a/..//evil.example",
            "/.//evil.example",
            "/%2e//evil.example",
            "/%2E%2e/x",
            "/.%2E",
            "/photos/.?q=1",
        ] {
            assert!(url_error(candidate).contains(". or .."), "{candidate}");
        }
    }

    #[test]
    fn dots_inside_a_segment_or_the_query_are_kept() {
        for candidate in [
            "/photos/a.b",
            "/photos/...",
            "/.well",
            "/search?q=..",
            "/search?q=/../x",
            "/folders?path=/a/./b",
        ] {
            assert!(url(candidate).is_ok(), "{candidate}");
        }
    }

    #[test]
    fn a_url_has_no_fragment() {
        assert!(url_error("/a#b").contains("fragment"));
        assert!(url_error("/#").contains("fragment"));
    }
}

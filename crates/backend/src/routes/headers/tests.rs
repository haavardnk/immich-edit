use super::*;

#[test]
fn attachment_names_are_quoted_and_utf8_encoded() {
    for (filename, expected) in [
        (
            "say \"hi\".jpg",
            "attachment; filename=\"say _hi_.jpg\"; filename*=UTF-8''say%20%22hi%22.jpg",
        ),
        (
            "fjørd.tif",
            "attachment; filename=\"fj_rd.tif\"; filename*=UTF-8''fj%C3%B8rd.tif",
        ),
        (
            "a\r\nb.jpg",
            "attachment; filename=\"a__b.jpg\"; filename*=UTF-8''a%0D%0Ab.jpg",
        ),
    ] {
        let value = attachment(filename).expect("attachment header");
        assert_eq!(value.to_str().unwrap(), expected, "{filename:?}");
    }
}

#[test]
fn json_lists_are_ascii_and_round_trip() {
    let items = vec![
        "Metadata not copied: fjørd 📷".to_string(),
        "line\r\nbreak \"quoted\"\u{7f}".to_string(),
    ];
    let value = json_list(&items).expect("json list header");
    let text = value.to_str().unwrap();
    assert!(text.is_ascii(), "{text}");
    assert_eq!(serde_json::from_str::<Vec<String>>(text).unwrap(), items);
}

#[test]
fn etags_are_quoted_and_matched_against_if_none_match() {
    let tag = etag("abc-400").expect("etag");
    assert_eq!(tag.to_str().unwrap(), "\"abc-400\"");
    let mut headers = HeaderMap::new();
    headers.insert(
        header::IF_NONE_MATCH,
        HeaderValue::from_static("\"other\", \"abc-400\""),
    );
    assert!(etag_matches(&headers, &tag));
    assert!(!etag_matches(&HeaderMap::new(), &tag));
}

#[test]
fn zstd_is_accepted_only_when_listed_with_a_positive_quality() {
    for (accept, expected) in [
        ("gzip, deflate, br, zstd", true),
        ("ZSTD;q=0.5", true),
        ("zstd ; Q=1", true),
        ("gzip, zstd;q=0", false),
        ("zstd;q=0.000", false),
        ("gzip, br", false),
        ("*", false),
        ("zstdx", false),
    ] {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT_ENCODING, HeaderValue::from_static(accept));
        assert_eq!(accepts_zstd(&headers), expected, "{accept:?}");
    }
    assert!(!accepts_zstd(&HeaderMap::new()));
}

#[test]
fn control_characters_are_rejected() {
    assert!(matches!(etag("a\rb"), Err(AppError::Internal)));
    assert!(matches!(
        header_value("image/\njpeg"),
        Err(AppError::Internal)
    ));
}

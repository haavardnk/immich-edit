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
fn control_characters_are_rejected() {
    assert!(matches!(etag("a\rb"), Err(AppError::Internal)));
    assert!(matches!(
        header_value("image/\njpeg"),
        Err(AppError::Internal)
    ));
}

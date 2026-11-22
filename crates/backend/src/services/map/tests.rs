use uuid::Uuid;

use super::*;
use crate::services::immich_capabilities::ImmichCapabilities;

fn caps(light: Option<&str>, dark: Option<&str>) -> ImmichCapabilities {
    ImmichCapabilities {
        version: None,
        structured_search: false,
        tag_rename: false,
        smart_search: true,
        ocr: true,
        trash: false,
        trash_days: None,
        external_host: None,
        map_light_style: light.map(Into::into),
        map_dark_style: dark.map(Into::into),
    }
}

fn settings(light: Option<&str>, dark: Option<&str>, geocoder: &str) -> MapSettings {
    MapSettings {
        style_light: light.map(Into::into),
        style_dark: dark.map(Into::into),
        geocoder_url: geocoder.into(),
    }
}

#[test]
fn settings_trim_urls_and_need_both_styles() {
    let ok = settings(
        Some(" https://tiles.example.net/light.json "),
        Some("https://tiles.example.net/dark.json"),
        "http://photon.lan:2322/",
    )
    .validated()
    .unwrap();
    assert_eq!(
        ok.style_light.as_deref(),
        Some("https://tiles.example.net/light.json")
    );
    assert_eq!(ok.geocoder_url, "http://photon.lan:2322");
    let blanks = settings(Some(" "), Some(""), "").validated().unwrap();
    assert_eq!(blanks, settings(None, None, ""));
    for bad in [
        settings(Some("https://a.example/l.json"), None, ""),
        settings(
            Some("ftp://a.example/l.json"),
            Some("https://a.example/d.json"),
            "",
        ),
        settings(None, None, "photon.komoot.io"),
        settings(
            None,
            None,
            &format!("https://a.example/{}", "x".repeat(2048)),
        ),
    ] {
        assert!(bad.clone().validated().is_err(), "{bad:?}");
    }
}

#[test]
fn custom_styles_win_then_immich_then_none() {
    let custom = settings(Some("https://c/l"), Some("https://c/d"), "");
    let immich = caps(Some("https://i/l"), Some("https://i/d"));
    let styles = resolve_styles(&custom, &immich);
    assert_eq!(
        (styles.light.as_deref(), styles.source),
        (Some("https://c/l"), StyleSource::Custom)
    );
    let styles = resolve_styles(&MapSettings::default(), &immich);
    assert_eq!(
        (styles.dark.as_deref(), styles.source),
        (Some("https://i/d"), StyleSource::Immich)
    );
    let one_sided = resolve_styles(&MapSettings::default(), &caps(Some("https://i/l"), None));
    assert_eq!(one_sided.dark.as_deref(), Some("https://i/l"));
    let none = resolve_styles(&MapSettings::default(), &caps(None, None));
    assert_eq!((none.light, none.source), (None, StyleSource::None));
}

#[test]
fn photon_features_become_named_places() {
    let body = br#"{"type":"FeatureCollection","features":[
      {"geometry":{"coordinates":[5.324,60.3975]},"properties":{"name":"Bryggen","city":"Bergen","county":"Vestland","state":"Vestland","country":"Norway"}},
      {"geometry":{"coordinates":[10.75,59.91]},"properties":{"street":"Karl Johans gate","housenumber":"1","city":"Oslo","country":"Norway"}},
      {"geometry":{"coordinates":[200.0,10.0]},"properties":{"name":"Nowhere"}},
      {"geometry":{"coordinates":[5.0,60.0]},"properties":{}}
    ]}"#;
    let places = photon::parse(body).unwrap();
    assert_eq!(
        places,
        vec![
            Place {
                name: "Bryggen".into(),
                detail: "Bergen, Vestland, Norway".into(),
                lat: 60.3975,
                lon: 5.324
            },
            Place {
                name: "Karl Johans gate 1".into(),
                detail: "Oslo, Norway".into(),
                lat: 59.91,
                lon: 10.75
            }
        ]
    );
}

#[tokio::test]
async fn search_refuses_when_off_or_the_query_is_unusable() {
    let geocoder = Geocoder::new();
    let owner = Uuid::nil();
    assert!(matches!(
        geocoder.search("", owner, "Bergen").await,
        Err(GeocodeError::Off)
    ));
    for query in [" b ", "Berg\u{7}en", &"x".repeat(121)] {
        assert!(matches!(
            geocoder
                .search("https://photon.example", owner, query)
                .await,
            Err(GeocodeError::BadQuery)
        ));
    }
}

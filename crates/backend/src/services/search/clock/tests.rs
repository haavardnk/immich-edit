use chrono::{NaiveDateTime, TimeDelta, TimeZone};
use serde_json::json;

use super::*;

fn at(raw: &str) -> DateTime<Utc> {
    parse_at(raw).unwrap()
}

fn wall(raw: &str) -> NaiveDateTime {
    at(raw).naive_utc()
}

#[test]
fn each_clock_reads_its_own_capture_time() {
    let asset: AssetSummary = serde_json::from_value(json!({
        "id": uuid::Uuid::nil(),
        "fileCreatedAt": "2025-03-18T21:30:00.000Z",
        "localDateTime": "2025-03-19T00:30:00.000Z",
    }))
    .unwrap();
    let read = [Clock::Utc.taken(&asset), Clock::Photo.taken(&asset)];
    if read
        != [
            Some(at("2025-03-18T21:30:00Z")),
            Some(at("2025-03-19T00:30:00Z")),
        ]
    {
        panic!("clocks read {read:?}");
    }
}

#[test]
fn a_bounded_filter_keeps_the_later_start_and_the_picked_end() {
    let march = Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap();
    let april = Utc.with_ymd_and_hms(2024, 4, 1, 0, 0, 0).unwrap();
    let picked_after = Utc.with_ymd_and_hms(2024, 3, 10, 0, 0, 0).unwrap();
    let picked_before = Utc.with_ymd_and_hms(2024, 3, 20, 0, 0, 0).unwrap();
    let cases = [
        (None, None, march),
        (Some(picked_after), Some(picked_before), picked_after),
        (Some(march - TimeDelta::days(9)), None, march),
    ];
    for (taken_after, taken_before, start) in cases {
        let filter = BrowseFilter {
            taken_after,
            taken_before,
            favorite: true,
            ..BrowseFilter::default()
        };
        let bounded = Clock::Utc.bound(&filter, (Some(march), Some(april)));
        let kept = (
            bounded.taken_after,
            bounded.taken_before,
            bounded.taken_until,
            bounded.favorite,
        );
        if kept != (Some(start), taken_before, Some(april), true) {
            panic!("bounded filter kept {kept:?}");
        }
    }
    let picked_until = Utc.with_ymd_and_hms(2024, 3, 15, 0, 0, 0).unwrap();
    for (until, kept) in [
        (picked_until, picked_until),
        (april + TimeDelta::days(3), april),
    ] {
        let filter = BrowseFilter {
            taken_until: Some(until),
            ..BrowseFilter::default()
        };
        let bounded = Clock::Utc
            .bound(&filter, (Some(march), Some(april)))
            .taken_until;
        if bounded != Some(kept) {
            panic!("a picked end of {until} was bounded to {bounded:?}");
        }
    }
}

#[test]
fn a_photo_bound_narrows_the_wall_clock_span_and_leaves_instants_alone() {
    let filter = BrowseFilter {
        taken_local: Some(LocalSpan {
            from: Some(wall("2025-03-10T00:00:00Z")),
            until: None,
        }),
        ..BrowseFilter::default()
    };
    let month = (
        Some(at("2025-03-01T00:00:00Z")),
        Some(at("2025-04-01T00:00:00Z")),
    );
    let bounded = Clock::Photo.bound(&filter, month);
    let expected = LocalSpan {
        from: Some(wall("2025-03-10T00:00:00Z")),
        until: Some(wall("2025-04-01T00:00:00Z")),
    };
    if (
        bounded.taken_local,
        bounded.taken_after,
        bounded.taken_until,
    ) != (Some(expected), None, None)
    {
        panic!("bounded to {bounded:?}");
    }
    let open = Clock::Photo.bound(
        &BrowseFilter::default(),
        (None, Some(at("2025-03-05T12:00:00Z"))),
    );
    let expected = LocalSpan {
        from: None,
        until: Some(wall("2025-03-05T12:00:00Z")),
    };
    if open.taken_local != Some(expected) {
        panic!("an open start bounded to {:?}", open.taken_local);
    }
}

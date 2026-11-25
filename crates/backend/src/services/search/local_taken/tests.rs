use chrono::NaiveDate;
use serde_json::json;

use super::*;

fn utc(text: &str) -> DateTime<Utc> {
    parse_at(text).unwrap()
}

fn local(text: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M").unwrap()
}

fn day(year: i32, month: u32, day: u32) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(year, month, day)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
}

fn span(from: Option<NaiveDateTime>, until: Option<NaiveDateTime>) -> LocalSpan {
    LocalSpan { from, until }
}

fn seen(at: &str, local_at: &str) -> Seen {
    Seen {
        id: Uuid::new_v4(),
        at: utc(at),
        local: local(local_at),
        added: Some(utc("2025-04-01T10:00:00Z")),
        updated: Some(utc("2025-04-02T10:00:00Z")),
    }
}

#[test]
fn each_end_is_its_own_slice_and_a_long_span_trusts_its_middle() {
    let one_day = span(Some(day(2025, 3, 18)), Some(day(2025, 3, 19))).plan();
    let slices = (one_day.sure, one_day.slices);
    let expected = (
        None,
        vec![
            (utc("2025-03-17T10:00:00Z"), utc("2025-03-18T12:00:00Z")),
            (utc("2025-03-18T10:00:00Z"), utc("2025-03-19T12:00:00Z")),
        ],
    );
    if slices != expected {
        panic!("one day planned {slices:?}");
    }

    let march = span(Some(day(2025, 3, 1)), Some(day(2025, 4, 1))).plan();
    let slices = (march.sure, march.slices);
    let expected = (
        Some((
            Some(utc("2025-03-01T12:00:00Z")),
            Some(utc("2025-03-31T10:00:00Z")),
        )),
        vec![
            (utc("2025-02-28T10:00:00Z"), utc("2025-03-01T12:00:00Z")),
            (utc("2025-03-31T10:00:00Z"), utc("2025-04-01T12:00:00Z")),
        ],
    );
    if slices != expected {
        panic!("March planned {slices:?}");
    }

    let open = span(Some(day(2025, 3, 1)), None).plan();
    let slices = (open.sure, open.slices);
    let expected = (
        Some((Some(utc("2025-03-01T12:00:00Z")), None)),
        vec![(utc("2025-02-28T10:00:00Z"), utc("2025-03-01T12:00:00Z"))],
    );
    if slices != expected {
        panic!("an open end planned {slices:?}");
    }
}

#[test]
fn photos_of_one_zone_tighten_the_bounds_and_exclude_nothing() {
    let picked = span(Some(day(2025, 3, 18)), Some(day(2025, 3, 19)));
    let early = seen("2025-03-17T22:00:00Z", "2025-03-18T01:00");
    let late = seen("2025-03-18T15:00:00Z", "2025-03-18T18:00");
    let next_night = seen("2025-03-18T21:30:00Z", "2025-03-19T00:30");
    let bounds = LocalBounds::settle(picked, &picked.plan(), &[early, late, next_night]);
    let expected = LocalBounds {
        after: Some(early.at),
        until: Some(late.at + TimeDelta::milliseconds(1)),
        excluded: Vec::new(),
    };
    if bounds.as_ref() != Some(&expected) {
        panic!("bounds were {bounds:?}");
    }
}

#[test]
fn a_photo_from_another_zone_inside_the_bounds_is_excluded() {
    let picked = span(Some(day(2025, 3, 18)), Some(day(2025, 3, 19)));
    let early = seen("2025-03-17T22:00:00Z", "2025-03-18T01:00");
    let late = seen("2025-03-18T15:00:00Z", "2025-03-18T18:00");
    let west = seen("2025-03-18T03:00:00Z", "2025-03-17T22:00");
    let bounds = LocalBounds::settle(picked, &picked.plan(), &[early, west, late]);
    let excluded = bounds.map(|bounds| bounds.excluded);
    if excluded != Some(vec![west]) {
        panic!("excluded {excluded:?}");
    }
}

#[test]
fn a_short_span_without_photos_is_empty_and_a_long_one_keeps_its_middle() {
    let picked = span(Some(day(2025, 3, 18)), Some(day(2025, 3, 19)));
    let outside = seen("2025-03-18T21:30:00Z", "2025-03-19T00:30");
    if let Some(bounds) = LocalBounds::settle(picked, &picked.plan(), &[outside]) {
        panic!("an empty day settled to {bounds:?}");
    }

    let march = span(Some(day(2025, 3, 1)), Some(day(2025, 4, 1)));
    let bounds = LocalBounds::settle(march, &march.plan(), &[]);
    let expected = LocalBounds {
        after: Some(utc("2025-03-01T12:00:00Z")),
        until: Some(utc("2025-03-31T10:00:00Z")),
        excluded: Vec::new(),
    };
    if bounds.as_ref() != Some(&expected) {
        panic!("March settled to {bounds:?}");
    }
}

#[test]
fn a_reversed_span_is_empty() {
    let reversed = span(Some(day(2025, 3, 19)), Some(day(2025, 3, 18)));
    let same = span(Some(day(2025, 3, 18)), Some(day(2025, 3, 18)));
    let open = span(Some(day(2025, 3, 18)), None);
    let empty = [reversed, same, open].map(LocalSpan::is_empty);
    if empty != [true, true, false] {
        panic!("emptiness was {empty:?}");
    }
}

#[test]
fn narrowing_keeps_the_tighter_bound_and_only_excluded_photos_inside_it() {
    let inside = seen("2025-03-18T03:00:00Z", "2025-03-17T22:00");
    let outside = seen("2025-03-18T12:00:00Z", "2025-03-19T00:00");
    let bounds = LocalBounds {
        after: Some(utc("2025-03-17T22:00:00Z")),
        until: Some(utc("2025-03-18T15:00:00Z")),
        excluded: vec![inside, outside],
    };
    let call = BrowseFilter {
        taken_local: Some(span(Some(day(2025, 3, 18)), Some(day(2025, 3, 19)))),
        taken_after: Some(utc("2025-03-18T00:00:00Z")),
        taken_until: Some(utc("2025-03-18T06:00:00Z")),
        favorite: true,
        ..BrowseFilter::default()
    };
    let narrowed = bounds.narrow(&call);
    let kept = (
        narrowed.taken_local,
        narrowed.taken_after,
        narrowed.taken_until,
        narrowed.favorite,
    );
    let expected = (
        None,
        Some(utc("2025-03-18T00:00:00Z")),
        Some(utc("2025-03-18T06:00:00Z")),
        true,
    );
    if kept != expected {
        panic!("narrowed to {kept:?}");
    }
    let excluded = bounds.excluded_by(&narrowed);
    if excluded != HashSet::from([inside.id]) {
        panic!("excluded {excluded:?}");
    }
    let unbounded = bounds.narrow(&BrowseFilter::default());
    if (unbounded.taken_after, unbounded.taken_until) != (bounds.after, bounds.until) {
        panic!("an unbounded call narrowed to {unbounded:?}");
    }
}

#[test]
fn a_call_narrowed_by_added_or_updated_time_only_excludes_photos_it_would_list() {
    let bounds = LocalBounds {
        after: None,
        until: None,
        excluded: vec![seen("2025-03-18T03:00:00Z", "2025-03-17T22:00")],
    };
    let cases = [
        (None, None, 1),
        (Some(utc("2025-04-01T10:00:00Z")), None, 1),
        (Some(utc("2025-04-01T10:00:01Z")), None, 0),
        (None, Some(utc("2025-04-02T10:00:00Z")), 1),
        (None, Some(utc("2025-04-02T10:00:01Z")), 0),
    ];
    for (added_from, updated_after, expected) in cases {
        let call = BrowseFilter {
            added_between: added_from.map(|from| (from, utc("2025-05-01T00:00:00Z"))),
            updated_after,
            ..BrowseFilter::default()
        };
        let excluded = bounds.excluded_by(&bounds.narrow(&call)).len();
        if excluded != expected {
            panic!("added {added_from:?} updated {updated_after:?} excluded {excluded}");
        }
    }
}

#[test]
fn a_photo_reads_its_local_clock_and_falls_back_to_its_instant() {
    let read = |local_date_time: Option<&str>| {
        let asset: AssetSummary = serde_json::from_value(json!({
            "id": Uuid::nil(),
            "fileCreatedAt": "2025-03-18T21:30:00.000Z",
            "localDateTime": local_date_time,
        }))
        .unwrap();
        Seen::of(&asset).map(|seen| seen.local)
    };
    let found = [read(Some("2025-03-19T00:30:00.000Z")), read(None)];
    let expected = [
        Some(local("2025-03-19T00:30")),
        Some(local("2025-03-18T21:30")),
    ];
    if found != expected {
        panic!("local clocks were {found:?}");
    }
}

#[test]
fn the_wire_span_reads_naive_local_times() {
    let read = serde_json::from_value::<BrowseFilter>(json!({
        "takenLocal": { "from": "2025-03-18T06:30:00", "until": "2025-03-19T00:00:00" }
    }))
    .ok()
    .and_then(|filter| filter.taken_local);
    if read
        != Some(span(
            Some(local("2025-03-18T06:30")),
            Some(day(2025, 3, 19)),
        ))
    {
        panic!("takenLocal read as {read:?}");
    }
    let unknown = serde_json::from_value::<BrowseFilter>(json!({
        "takenLocal": { "from": "2025-03-18T06:30:00", "zone": "UTC" }
    }));
    if unknown.is_ok() {
        panic!("an unknown takenLocal key was accepted");
    }
}

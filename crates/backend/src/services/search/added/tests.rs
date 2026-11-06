mod fake;

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, TimeZone, Utc};
use uuid::Uuid;

use super::*;
use crate::services::search::filter::Visibility;
use crate::services::search::query::{Sort, SortField};
use crate::services::search::window;
use fake::{Asset, HIDDEN_PARTNER, PARTNER, Pick, library, requests, serve};
use timeline::SCAN_CHUNKS;

fn images() -> BrowseFilter {
    BrowseFilter {
        images_only: true,
        ..BrowseFilter::default()
    }
}

fn favorites() -> BrowseFilter {
    BrowseFilter {
        favorite: true,
        visibility: Some(Visibility::Timeline),
        images_only: true,
        ..BrowseFilter::default()
    }
}

fn added(direction: Direction) -> Sort {
    Sort {
        field: SortField::Added,
        direction,
    }
}

fn picked_in_order(assets: &[Asset], direction: Direction) -> Vec<Uuid> {
    let mut picked: Vec<(DateTime<Utc>, Uuid)> = assets
        .iter()
        .filter(|asset| asset.picked)
        .map(|asset| (asset.at, asset.id))
        .collect();
    picked.sort();
    if direction == Direction::Desc {
        picked.reverse();
    }
    picked.into_iter().map(|(_, id)| id).collect()
}

async fn feed(
    searcher: &Searcher,
    filter: &BrowseFilter,
    direction: Direction,
    size: u32,
) -> (Vec<Uuid>, usize) {
    let mut query = PageQuery {
        filter: filter.clone(),
        sort: added(direction),
        size,
        cursor: None,
        with_exif: false,
    };
    let mut ids = Vec::new();
    let mut empty = 0;
    loop {
        let page = searcher.page(&query).await.unwrap();
        if page.items.is_empty() && page.next_cursor.is_some() {
            empty += 1;
        }
        ids.extend(page.items.iter().map(|asset| asset.id.source()));
        let Some(cursor) = page.next_cursor else {
            return (ids, empty);
        };
        query.cursor = Some(cursor);
    }
}

async fn check_windows(searcher: &Searcher, filter: &BrowseFilter, feed: &[Uuid], every: usize) {
    let direction = Direction::Desc;
    let keyed: Vec<((), Uuid)> = feed.iter().map(|id| ((), *id)).collect();
    let anchors = (0..feed.len()).filter(|index| index % every == 0 || *index + 1 == feed.len());
    for index in anchors {
        for reach in window::tests::reaches() {
            let found = window::around(searcher, filter, added(direction), feed[index], reach)
                .await
                .unwrap();
            let expected = window::tests::expected_window(&keyed, index, reach);
            if window::tests::outline(&found) != expected {
                panic!("window around {index} with {reach:?} was {found:?}, expected {expected:?}");
            }
        }
    }
}

fn shared_with_partner(count: usize, pick: Pick) -> Vec<Asset> {
    let mut assets = library(count, pick);
    for index in (1..count).step_by(2) {
        assets[index].partner = true;
        assets[index].at = assets[index - 1].at;
    }
    assets
}

#[tokio::test]
async fn the_added_feed_walks_the_timeline_through_the_filter() {
    let cases: [(Pick, u32, bool); 3] = [
        (|i| i % 3 == 0, 3, false),
        (|i| i % 3 == 0, 1000, false),
        (|i| i == 0, 1, true),
    ];
    for (pick, size, sparse) in cases {
        let assets = library(40, pick);
        let (_server, searcher) = serve(&assets, true).await;
        for direction in [Direction::Asc, Direction::Desc] {
            let wanted = picked_in_order(&assets, direction);
            let (ids, empty) = feed(&searcher, &images(), direction, size).await;
            if ids != wanted {
                panic!("{direction:?} size {size} feed was {ids:?}, expected {wanted:?}");
            }
            if sparse && direction == Direction::Desc && empty == 0 {
                panic!("a sparse filter should hand back empty pages that keep a cursor");
            }
        }
    }
}

#[tokio::test]
async fn an_added_range_walks_only_the_months_it_covers() {
    let assets = library(40, |_| true);
    let (server, searcher) = serve(&assets, true).await;
    let after = Utc.with_ymd_and_hms(2024, 2, 10, 0, 0, 0).unwrap();
    let before = Utc.with_ymd_and_hms(2024, 3, 5, 0, 0, 0).unwrap();
    let mut expected: Vec<(DateTime<Utc>, Uuid)> = assets
        .iter()
        .filter(|asset| asset.at >= after && asset.at <= before)
        .map(|asset| (asset.at, asset.id))
        .collect();
    expected.sort();
    let query = PageQuery {
        filter: BrowseFilter {
            added_after: Some(after),
            added_before: Some(before),
            ..BrowseFilter::default()
        },
        sort: added(Direction::Asc),
        size: 1000,
        cursor: None,
        with_exif: false,
    };
    let page = searcher.page(&query).await.unwrap();
    let ids: Vec<Uuid> = page.items.iter().map(|asset| asset.id.source()).collect();
    let wanted: Vec<Uuid> = expected.iter().map(|(_, id)| *id).collect();
    if ids != wanted {
        panic!("added range gave {ids:?}, expected {wanted:?}");
    }
    let mut walked: Vec<String> = requests(&server, "/api/timeline/bucket")
        .await
        .into_iter()
        .filter_map(|query| query.get("timeBucket").cloned())
        .collect();
    walked.dedup();
    if walked != ["2024-02-01", "2024-03-01"] {
        panic!("added range walked {walked:?}");
    }
}

#[tokio::test]
async fn the_added_window_matches_the_feed_around_the_anchor() {
    let assets = library(40, |i| i % 2 == 0);
    let (_server, searcher) = serve(&assets, true).await;
    for filter in [images(), favorites()] {
        for direction in [Direction::Desc, Direction::Asc] {
            let (feed, _) = feed(&searcher, &filter, direction, 50).await;
            let sort = added(direction);
            for index in [0, 1, 7, feed.len() - 2, feed.len() - 1] {
                for reach in window::tests::reaches() {
                    let found = window::around(&searcher, &filter, sort, feed[index], reach)
                        .await
                        .unwrap();
                    let keyed: Vec<((), Uuid)> = feed.iter().map(|id| ((), *id)).collect();
                    let expected = window::tests::expected_window(&keyed, index, reach);
                    if window::tests::outline(&found) != expected {
                        panic!(
                            "{direction:?} window around {index} with {reach:?} was {found:?}, expected {expected:?}"
                        );
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn a_sparse_added_window_scans_on_until_it_knows_what_lies_beyond() {
    let count = SIDE_CHUNK * SCAN_CHUNKS + 40;
    let assets = library(count, |i| i < 2 || i == 1063);
    let (server, searcher) = serve(&assets, true).await;
    let id = |index: usize| assets[index].id;
    let cases = [
        (0, 0, (vec![id(1)], true, true)),
        (1, 0, (vec![id(1063), id(1)], false, true)),
        (2, 2, (vec![id(1063), id(1), id(0)], false, false)),
    ];
    for (before, after, expected) in cases.clone() {
        let reach = Reach { before, after };
        let found = window::around(&searcher, &images(), added(Direction::Desc), id(1), reach)
            .await
            .unwrap();
        if window::tests::outline(&found) != expected {
            panic!("sparse window with {reach:?} was {found:?}, expected {expected:?}");
        }
    }
    let lists = requests(&server, "/api/timeline/buckets").await.len();
    if lists != cases.len() {
        panic!(
            "{} windows loaded the bucket list {lists} times",
            cases.len()
        );
    }
}

#[tokio::test]
async fn favorites_walk_each_owner_timeline_narrowed_to_favorites() {
    let assets = shared_with_partner(48, |i| i % 3 != 1);
    let (server, searcher) = serve(&assets, true).await;
    let wanted: HashSet<Uuid> = assets
        .iter()
        .filter(|asset| asset.picked)
        .map(|asset| asset.id)
        .collect();
    let at: HashMap<Uuid, DateTime<Utc>> =
        assets.iter().map(|asset| (asset.id, asset.at)).collect();
    for direction in [Direction::Desc, Direction::Asc] {
        let (ids, _) = feed(&searcher, &favorites(), direction, 7).await;
        let found: HashSet<Uuid> = ids.iter().copied().collect();
        let ordered = ids.windows(2).all(|pair| {
            let (a, b) = (at[&pair[0]], at[&pair[1]]);
            if direction == Direction::Desc {
                a >= b
            } else {
                a <= b
            }
        });
        if found != wanted || ids.len() != wanted.len() || !ordered {
            panic!("{direction:?} favorites feed was {ids:?}");
        }
    }
    let (desc, _) = feed(&searcher, &favorites(), Direction::Desc, 50).await;
    check_windows(&searcher, &favorites(), &desc, 5).await;
    let queries = requests(&server, "/api/timeline/buckets").await;
    let narrowed =
        |query: &fake::Query| query.get("isFavorite").map(String::as_str) == Some("true");
    let owner = |query: &fake::Query| query.get("userId").cloned();
    let own = queries
        .iter()
        .any(|query| narrowed(query) && owner(query).is_none());
    let partner = queries
        .iter()
        .any(|query| narrowed(query) && owner(query) == Some(PARTNER.to_string()));
    let unnarrowed = queries.iter().any(|query| !narrowed(query));
    let hidden = queries
        .iter()
        .any(|query| owner(query) == Some(HIDDEN_PARTNER.to_string()));
    let shared = queries
        .iter()
        .any(|query| query.contains_key("withPartners"));
    if !own || !partner || unnarrowed || hidden || shared {
        panic!("favorites asked the timeline for {queries:?}");
    }
}

#[tokio::test]
async fn a_key_without_partner_access_keeps_the_shared_timeline() {
    let assets = shared_with_partner(24, |i| i % 3 != 1);
    let (server, searcher) = serve(&assets, false).await;
    let (ids, _) = feed(&searcher, &favorites(), Direction::Desc, 5).await;
    let found: HashSet<Uuid> = ids.iter().copied().collect();
    let wanted: HashSet<Uuid> = assets
        .iter()
        .filter(|asset| asset.picked)
        .map(|asset| asset.id)
        .collect();
    if found != wanted {
        panic!("favorites without partner access gave {ids:?}");
    }
    let queries = requests(&server, "/api/timeline/buckets").await;
    let shared = queries.iter().all(|query| {
        query.get("withPartners").map(String::as_str) == Some("true")
            && !query.contains_key("isFavorite")
    });
    if queries.is_empty() || !shared {
        panic!("without partner access the timeline was asked for {queries:?}");
    }
}

#[tokio::test]
async fn a_sparse_favorites_window_reads_only_favorites() {
    let count = 2000;
    let assets = library(count, |i| i < 2 || i == 1999);
    let (server, searcher) = serve(&assets, true).await;
    let reach = Reach {
        before: 100,
        after: 100,
    };
    let found = window::around(
        &searcher,
        &favorites(),
        added(Direction::Desc),
        assets[1].id,
        reach,
    )
    .await
    .unwrap();
    let expected = (
        vec![assets[1999].id, assets[1].id, assets[0].id],
        false,
        false,
    );
    if window::tests::outline(&found) != expected {
        panic!("sparse favorites window was {found:?}, expected {expected:?}");
    }
    let searches = requests(&server, "/api/search/metadata").await.len();
    let lists = requests(&server, "/api/timeline/buckets").await.len();
    if searches > 3 || lists != 2 {
        panic!("sparse favorites window made {searches} searches and {lists} bucket lists");
    }
}
